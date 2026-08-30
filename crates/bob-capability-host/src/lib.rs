use bob_core::planning::{project_remaining_work, PlanProjection, PlanningRequest};
use bob_core::validation::validate_planning_request;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const PLAN_METHOD: &str = "planRemainingWork";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolRequest {
    protocol_version: u32,
    request_id: String,
    method: String,
    params: PlanningRequest,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolResponse {
    protocol_version: u32,
    request_id: Option<String>,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<PlanProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ProtocolError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProtocolError {
    code: &'static str,
    message: String,
}

pub fn handle_line(line: &str) -> String {
    if line.len() > MAX_MESSAGE_BYTES {
        return encode_error(None, "message_too_large", "request exceeds the 64 KiB protocol limit");
    }

    let request = match serde_json::from_str::<ProtocolRequest>(line) {
        Ok(request) => request,
        Err(_) => return encode_error(None, "invalid_json", "request is not valid protocol JSON"),
    };

    let request_id = Some(request.request_id.clone());
    if request.request_id.trim().is_empty() || request.request_id.len() > 128 {
        return encode_error(
            request_id,
            "invalid_request_id",
            "requestId must be between 1 and 128 characters",
        );
    }
    if request.protocol_version != PROTOCOL_VERSION {
        return encode_error(
            request_id,
            "unsupported_protocol",
            format!(
                "protocolVersion {} is unsupported; expected {}",
                request.protocol_version, PROTOCOL_VERSION
            ),
        );
    }
    if request.method != PLAN_METHOD {
        return encode_error(
            request_id,
            "unsupported_method",
            format!("method {} is unsupported", request.method),
        );
    }
    if let Err(message) = validate_planning_request(&request.params) {
        return encode_error(request_id, "invalid_params", message);
    }

    let result = project_remaining_work(&request.params);
    encode_response(ProtocolResponse {
        protocol_version: PROTOCOL_VERSION,
        request_id,
        ok: true,
        result: Some(result),
        error: None,
    })
}

fn encode_error(
    request_id: Option<String>,
    code: &'static str,
    message: impl Into<String>,
) -> String {
    encode_response(ProtocolResponse {
        protocol_version: PROTOCOL_VERSION,
        request_id,
        ok: false,
        result: None,
        error: Some(ProtocolError {
            code,
            message: message.into(),
        }),
    })
}

fn encode_response(response: ProtocolResponse) -> String {
    serde_json::to_string(&response).expect("protocol response serialization is infallible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn rejects_malformed_and_unsupported_requests_without_panicking() {
        let malformed: Value = serde_json::from_str(&handle_line("not json")).unwrap();
        assert_eq!(malformed["ok"], false);
        assert_eq!(malformed["error"]["code"], "invalid_json");

        let unsupported = r#"{"protocolVersion":9,"requestId":"r1","method":"planRemainingWork","params":{"activeId":null,"items":[]}}"#;
        let response: Value = serde_json::from_str(&handle_line(unsupported)).unwrap();
        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "unsupported_protocol");
    }
}
