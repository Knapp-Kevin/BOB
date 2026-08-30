use crate::planning::PlanningRequest;
use std::collections::HashSet;

const MAX_PLANNING_ITEMS: usize = 1_000;

pub fn validate_planning_request(request: &PlanningRequest) -> Result<(), &'static str> {
    if request.items.len() > MAX_PLANNING_ITEMS {
        return Err("planning request contains too many items");
    }

    let mut ids = HashSet::with_capacity(request.items.len());
    for item in &request.items {
        if item.id.trim().is_empty() || item.id.len() > 128 {
            return Err("planning item id is missing or too long");
        }
        if !ids.insert(item.id.as_str()) {
            return Err("planning request contains duplicate item ids");
        }
        if !matches!(item.kind.as_str(), "task" | "idea" | "note" | "reminder") {
            return Err("planning item kind is invalid");
        }
        if !matches!(item.priority.as_str(), "low" | "normal" | "high") {
            return Err("planning item priority is invalid");
        }
        if !matches!(
            item.status.as_str(),
            "inbox" | "planned" | "doing" | "done" | "deferred"
        ) {
            return Err("planning item status is invalid");
        }
        if item.due.as_ref().is_some_and(|due| due.len() > 200) {
            return Err("planning item due value is too long");
        }
    }

    if let Some(active_id) = request.active_id.as_deref() {
        if active_id.trim().is_empty() || active_id.len() > 128 {
            return Err("active planning item id is missing or too long");
        }
        if !ids.contains(active_id) {
            return Err("active planning item does not exist in request");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::PlanningItem;

    fn item(id: &str, status: &str, priority: &str) -> PlanningItem {
        PlanningItem {
            id: id.into(),
            kind: "task".into(),
            priority: priority.into(),
            due: None,
            status: status.into(),
        }
    }

    #[test]
    fn accepts_bounded_valid_input() {
        let request = PlanningRequest {
            active_id: Some("current".into()),
            items: vec![item("current", "planned", "normal")],
        };

        assert_eq!(validate_planning_request(&request), Ok(()));
    }

    #[test]
    fn rejects_duplicate_and_unknown_external_values() {
        let duplicate = PlanningRequest {
            active_id: None,
            items: vec![
                item("same", "planned", "normal"),
                item("same", "planned", "normal"),
            ],
        };
        assert_eq!(
            validate_planning_request(&duplicate),
            Err("planning request contains duplicate item ids")
        );

        let mut invalid = item("bad", "planned", "normal");
        invalid.priority = "urgent".into();
        let request = PlanningRequest {
            active_id: None,
            items: vec![invalid],
        };
        assert_eq!(
            validate_planning_request(&request),
            Err("planning item priority is invalid")
        );
    }
}
