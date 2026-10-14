use super::*;

/// How screens and exports show a rolling rollout checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutCheckpointView {
    pub code_text: String,
    pub label_text: String,
    pub margin: String,
    pub attempts: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_rolling_rollout_checkpoint_view(item: &RollingRolloutCheckpoint) -> RollingRolloutCheckpointView {
    RollingRolloutCheckpointView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        attempts: item.attempts.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_rollout_checkpoint_views(items: &[RollingRolloutCheckpoint]) -> Vec<RollingRolloutCheckpointView> {
    let mut views: Vec<RollingRolloutCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_rollout_checkpoint_view(item));
    }
    views
}

pub fn sum_rolling_rollout_checkpoint_view(view: &RollingRolloutCheckpointView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_margin + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_rollout_checkpoint("code-260", "label-525");
        let view = to_rolling_rollout_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_rollout_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
