use crate::state::{Store, WorkState};
use serde::Serialize;
use tauri::State;

#[path = "../../crates/bob-core/src/planning.rs"]
mod portable_planning;

pub use portable_planning::PlanProjection;
use portable_planning::{
    project_remaining_work as project_portable_work, PlanningItem, PlanningRequest,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplanResult {
    pub work_state: WorkState,
    pub plan: PlanProjection,
}

pub fn project_remaining_work(state: &WorkState) -> PlanProjection {
    let request = PlanningRequest {
        active_id: state.active_id.clone(),
        items: state
            .items
            .iter()
            .map(|item| PlanningItem {
                id: item.id.clone(),
                kind: item.kind.clone(),
                priority: item.priority.clone(),
                due: item.due.clone(),
                status: item.status.clone(),
            })
            .collect(),
    };

    project_portable_work(&request)
}

#[tauri::command]
pub fn plan_remaining_work(store: State<'_, Store>) -> std::result::Result<PlanProjection, String> {
    let state = store.load().map_err(|error| error.to_string())?;
    Ok(project_remaining_work(&state))
}

#[tauri::command]
pub fn replan_remaining_work(store: State<'_, Store>) -> std::result::Result<ReplanResult, String> {
    let mut state = store.load().map_err(|error| error.to_string())?;
    let plan = project_remaining_work(&state);
    state.active_id = plan.next_id.clone();
    store.save(&state).map_err(|error| error.to_string())?;

    Ok(ReplanResult {
        work_state: state,
        plan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{HandoffSnapshot, WorkItem};

    fn item(id: &str, status: &str, priority: &str, due: Option<&str>) -> WorkItem {
        WorkItem {
            id: id.into(),
            kind: "task".into(),
            title: format!("Task {id}"),
            estimate: Some(15),
            priority: priority.into(),
            due: due.map(str::to_owned),
            status: status.into(),
        }
    }

    fn state(active_id: Option<&str>, items: Vec<WorkItem>) -> WorkState {
        WorkState {
            active_id: active_id.map(str::to_owned),
            items,
            handoff: Some(HandoffSnapshot {
                objective: "Existing objective".into(),
                state: "Ready".into(),
                next: "Continue existing work".into(),
            }),
        }
    }

    #[test]
    fn preserves_current_eligible_active_item_first() {
        let state = state(
            Some("current"),
            vec![
                item("urgent", "planned", "high", Some("Today")),
                item("current", "planned", "low", None),
            ],
        );

        let plan = project_remaining_work(&state);

        assert_eq!(plan.next_id.as_deref(), Some("current"));
        assert_eq!(plan.focus_ids, vec!["current", "urgent"]);
    }

    #[test]
    fn excludes_completed_deferred_inbox_and_non_task_work_and_caps_focus() {
        let mut note = item("note", "planned", "high", Some("Today"));
        note.kind = "note".into();
        let state = state(
            Some("done"),
            vec![
                item("done", "done", "high", Some("Today")),
                item("deferred", "deferred", "high", Some("Today")),
                item("inbox", "inbox", "high", Some("Today")),
                note,
                item("doing", "doing", "normal", None),
                item("today", "planned", "high", Some("Today")),
                item("normal", "planned", "normal", None),
                item("low", "planned", "low", None),
            ],
        );

        let plan = project_remaining_work(&state);

        assert_eq!(plan.next_id.as_deref(), Some("doing"));
        assert_eq!(plan.focus_ids, vec!["doing", "today", "normal"]);
    }

    #[test]
    fn projection_does_not_mutate_canonical_work() {
        let state = state(
            None,
            vec![
                item("first", "planned", "high", Some("Today")),
                item("second", "planned", "normal", None),
            ],
        );
        let before = state.clone();

        let _ = project_remaining_work(&state);

        assert_eq!(state, before);
    }
}
