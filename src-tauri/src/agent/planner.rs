//! Planning phase (L2). Before acting, the agent asks the model to decompose the
//! task into an ordered list of steps. Parsing is deliberately lenient: if the
//! model doesn't return clean JSON, we fall back to a single step (the task
//! itself) so the run always proceeds.

use serde_json::json;
use shared::ipc::{PlanStatus, PlanStep};

use super::{AgentContext, llm};
use crate::util::new_id;

const PLANNER_SYSTEM: &str = "You are a planning module. Break the user's task into \
a short ordered list of concrete steps (1-5). Respond with ONLY a JSON array of \
strings, e.g. [\"step one\", \"step two\"]. No prose.";

pub async fn make_plan(ctx: &AgentContext, task: &str) -> Vec<PlanStep> {
    let messages = vec![
        json!({ "role": "system", "content": PLANNER_SYSTEM }),
        json!({ "role": "user", "content": task }),
    ];

    let descriptions = match llm::call(ctx, &messages, None, 0.2).await {
        Ok(reply) => parse_steps(&reply.content).unwrap_or_else(|| vec![task.to_string()]),
        Err(e) => {
            tracing::warn!(%e, "planner call failed; using single-step plan");
            vec![task.to_string()]
        }
    };

    descriptions
        .into_iter()
        .map(|d| PlanStep {
            id: new_id("step"),
            description: d,
            status: PlanStatus::Pending,
        })
        .collect()
}

/// Extract a JSON string array from the model's reply, tolerating surrounding
/// prose or code fences.
fn parse_steps(content: &str) -> Option<Vec<String>> {
    let start = content.find('[')?;
    let end = content.rfind(']')?;
    if end <= start {
        return None;
    }
    let arr: Vec<String> = serde_json::from_str(&content[start..=end]).ok()?;
    let arr: Vec<String> = arr
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if arr.is_empty() { None } else { Some(arr) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_array() {
        let got = parse_steps(r#"["a","b","c"]"#).unwrap();
        assert_eq!(got, vec!["a", "b", "c"]);
    }

    #[test]
    fn parses_array_with_prose_and_fences() {
        let got = parse_steps("Here is the plan:\n```json\n[\"one\", \"two\"]\n```").unwrap();
        assert_eq!(got, vec!["one", "two"]);
    }

    #[test]
    fn returns_none_on_garbage() {
        assert!(parse_steps("no array here").is_none());
    }
}
