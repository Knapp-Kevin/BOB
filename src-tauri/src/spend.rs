//! B.O.B.-side monthly spending limits for metered cloud inference.
//!
//! Every metered provider needs a user-set monthly limit (the UI pre-fills
//! $20). Amounts are integer micro-dollars. Before a request, B.O.B. reserves
//! the worst-case cost and refuses if that could exceed the limit; afterwards
//! it replaces the reservation with the actual cost. Months are UTC calendar
//! months.
//!
//! Worst case for OpenAI and Anthropic is exact, not estimated: byte-level
//! tokenizers emit at most one token per prompt byte, and output is capped at
//! `MAX_OUTPUT_TOKENS`. OpenRouter reports the exact cost of each reply but
//! not its price beforehand, so it is checked against the remaining limit and
//! can overshoot by at most one bounded request.

use std::time::{SystemTime, UNIX_EPOCH};

pub const MICROS_PER_DOLLAR: i64 = 1_000_000;
/// Pre-filled monthly limit shown during setup.
pub const DEFAULT_MONTHLY_CAP_MICROS: i64 = 20 * MICROS_PER_DOLLAR;
const MAX_MONTHLY_CAP_MICROS: i64 = 10_000 * MICROS_PER_DOLLAR;
const MAX_PRICE_MICROS_PER_MILLION: i64 = 1_000 * MICROS_PER_DOLLAR;
/// Per-message framing tokens added on top of the prompt bytes.
const MESSAGE_OVERHEAD_TOKENS: u64 = 64;

/// The current UTC month as `YYYY-MM`.
pub fn current_month() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    month_of(seconds)
}

fn month_of(unix_seconds: u64) -> String {
    // Civil-from-days (Howard Hinnant), valid for all dates after 1970.
    let days = (unix_seconds / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}")
}

/// Converts a user-entered dollar amount to micro-dollars.
pub fn dollars_to_micros(dollars: f64) -> Option<i64> {
    if !dollars.is_finite() || dollars <= 0.0 {
        return None;
    }
    let micros = (dollars * MICROS_PER_DOLLAR as f64).round();
    (micros >= 1.0 && micros <= i64::MAX as f64).then_some(micros as i64)
}

pub fn validate_cap(cap_micros: i64) -> bool {
    (1..=MAX_MONTHLY_CAP_MICROS).contains(&cap_micros)
}

pub fn validate_price(price_micros_per_million: i64) -> bool {
    (1..=MAX_PRICE_MICROS_PER_MILLION).contains(&price_micros_per_million)
}

/// Token prices for one approved model, in micro-dollars per million tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPrices {
    pub input: i64,
    pub output: i64,
}

impl TokenPrices {
    /// Cost of a request, rounded up to the next micro-dollar.
    pub fn cost(self, input_tokens: u64, output_tokens: u64) -> i64 {
        let total = u128::from(input_tokens) * self.input as u128
            + u128::from(output_tokens) * self.output as u128;
        i64::try_from(total.div_ceil(1_000_000)).unwrap_or(i64::MAX)
    }

    /// Upper bound on the cost of one request with this prompt.
    pub fn worst_case(self, prompt_bytes: usize, max_output_tokens: u32) -> i64 {
        self.cost(
            prompt_bytes as u64 + MESSAGE_OVERHEAD_TOKENS,
            u64::from(max_output_tokens),
        )
    }
}

/// Converts a provider-reported USD cost to micro-dollars, rounding up.
pub fn reported_cost_to_micros(cost: f64) -> Option<i64> {
    if !cost.is_finite() || cost < 0.0 {
        return None;
    }
    let micros = (cost * MICROS_PER_DOLLAR as f64).ceil();
    (micros <= i64::MAX as f64).then_some(micros as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_keys_are_utc_calendar_months() {
        assert_eq!(month_of(0), "1970-01");
        assert_eq!(month_of(951_782_400), "2000-02");
        // 2026-11-30T23:59:59Z and 2026-12-01T00:00:00Z
        assert_eq!(month_of(1_796_083_199), "2026-11");
        assert_eq!(month_of(1_796_083_200), "2026-12");
        assert_eq!(month_of(1_798_761_600), "2027-01");
        assert_eq!(current_month().len(), 7);
    }

    #[test]
    fn worst_case_bounds_any_reply_and_rounds_up() {
        let prices = TokenPrices {
            input: 3 * MICROS_PER_DOLLAR,
            output: 15 * MICROS_PER_DOLLAR,
        };
        // 4,000 prompt bytes + 64 overhead at $3/M, 512 output at $15/M.
        assert_eq!(prices.worst_case(4_000, 512), 12_192 + 7_680);
        assert!(prices.cost(10, 20) <= prices.worst_case(10, 512));
        assert_eq!(
            TokenPrices {
                input: 1,
                output: 1
            }
            .cost(1, 0),
            1,
            "rounds up"
        );
        assert_eq!(prices.cost(0, 0), 0);
    }

    #[test]
    fn dollar_inputs_are_validated() {
        assert_eq!(dollars_to_micros(20.0), Some(DEFAULT_MONTHLY_CAP_MICROS));
        assert_eq!(dollars_to_micros(0.25), Some(250_000));
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(dollars_to_micros(invalid), None);
        }
        assert!(validate_cap(DEFAULT_MONTHLY_CAP_MICROS));
        assert!(!validate_cap(0));
        assert!(!validate_cap(10_001 * MICROS_PER_DOLLAR));
        assert!(validate_price(150_000));
        assert!(!validate_price(1_001 * MICROS_PER_DOLLAR));
        assert_eq!(reported_cost_to_micros(0.0000001), Some(1));
        assert_eq!(reported_cost_to_micros(0.0), Some(0));
        assert_eq!(reported_cost_to_micros(-0.1), None);
        assert_eq!(reported_cost_to_micros(f64::NAN), None);
    }
}
