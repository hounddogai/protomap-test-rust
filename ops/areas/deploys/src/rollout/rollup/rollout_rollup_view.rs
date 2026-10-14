use super::*;

use acme_kit::text;

/// How screens and exports show a rollout rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRollupView {
    pub code_text: String,
    pub label: String,
    pub retries: String,
    pub attempts: String,
    pub external_ref_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_rollout_rollup_view(item: &RolloutRollup) -> RolloutRollupView {
    RolloutRollupView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        attempts: item.attempts.to_string(),
        external_ref_text: text::truncate(&item.external_ref, 80),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_rollup_views(items: &[RolloutRollup]) -> Vec<RolloutRollupView> {
    let mut views: Vec<RolloutRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_rollup_view(item));
    }
    views
}

pub fn sum_rollout_rollup_view(view: &RolloutRollupView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_attempts as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_rollup("code-963", "label-518");
        let view = to_rollout_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
