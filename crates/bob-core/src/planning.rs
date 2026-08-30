use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_FOCUS_ITEMS: usize = 3;
const MAX_PLANNING_ITEMS: usize = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningItem {
    pub id: String,
    pub kind: String,
    pub priority: String,
    pub due: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningRequest {
    pub active_id: Option<String>,
    pub items: Vec<PlanningItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanProjection {
    pub next_id: Option<String>,
    pub focus_ids: Vec<String>,
}

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

pub fn project_remaining_work(request: &PlanningRequest) -> PlanProjection {
    let active_id = request.active_id.as_deref();
    let mut candidates = request
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            item.kind == "task" && matches!(item.status.as_str(), "doing" | "planned")
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|(index, item)| {
        let active_rank = if active_id == Some(item.id.as_str()) {
            0
        } else {
            1
        };
        let status_rank = if item.status == "doing" { 0 } else { 1 };
        let today_rank = if item
            .due
            .as_deref()
            .is_some_and(|due| due.eq_ignore_ascii_case("today"))
        {
            0
        } else {
            1
        };
        let priority_rank = match item.priority.as_str() {
            "high" => 0,
            "normal" => 1,
            "low" => 2,
            _ => 3,
        };
        (active_rank, status_rank, today_rank, priority_rank, *index)
    });

    let focus_ids = candidates
        .iter()
        .take(MAX_FOCUS_ITEMS)
        .map(|(_, item)| item.id.clone())
        .collect::<Vec<_>>();

    PlanProjection {
        next_id: focus_ids.first().cloned(),
        focus_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, status: &str, priority: &str, due: Option<&str>) -> PlanningItem {
        PlanningItem {
            id: id.into(),
            kind: "task".into(),
            priority: priority.into(),
            due: due.map(str::to_owned),
            status: status.into(),
        }
    }

    #[test]
    fn preserves_active_work_then_orders_and_caps_remaining_focus() {
        let request = PlanningRequest {
            active_id: Some("current".into()),
            items: vec![
                item("urgent", "planned", "high", Some("Today")),
                item("current", "planned", "low", None),
                item("doing", "doing", "normal", None),
                item("normal", "planned", "normal", None),
                item("low", "planned", "low", None),
            ],
        };

        validate_planning_request(&request).expect("request should be valid");
        let plan = project_remaining_work(&request);

        assert_eq!(plan.next_id.as_deref(), Some("current"));
        assert_eq!(plan.focus_ids, vec!["current", "doing", "urgent"]);
    }

    #[test]
    fn ignores_non_task_and_ineligible_statuses() {
        let request = PlanningRequest {
            active_id: Some("done".into()),
            items: vec![
                item("done", "done", "high", Some("Today")),
                item("deferred", "deferred", "high", Some("Today")),
                item("inbox", "inbox", "high", Some("Today")),
                PlanningItem {
                    id: "note".into(),
                    kind: "note".into(),
                    priority: "high".into(),
                    due: Some("Today".into()),
                    status: "planned".into(),
                },
                item("eligible", "planned", "normal", None),
            ],
        };

        validate_planning_request(&request).expect("request should be valid");
        let plan = project_remaining_work(&request);

        assert_eq!(plan.next_id.as_deref(), Some("eligible"));
        assert_eq!(plan.focus_ids, vec!["eligible"]);
    }

    #[test]
    fn rejects_duplicate_and_unknown_external_values() {
        let duplicate = PlanningRequest {
            active_id: None,
            items: vec![
                item("same", "planned", "normal", None),
                item("same", "planned", "normal", None),
            ],
        };
        assert_eq!(
            validate_planning_request(&duplicate),
            Err("planning request contains duplicate item ids")
        );

        let mut invalid = item("bad", "planned", "normal", None);
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
