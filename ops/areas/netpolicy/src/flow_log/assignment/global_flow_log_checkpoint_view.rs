use super::*;

use acme_kit::text;

/// How screens and exports show a global flow log checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpointView {
    pub code_text: String,
    pub label: String,
    pub rotation_days: String,
    pub pinned: String,
    pub floor_value: String,
    pub batch_size: String,
    pub key_id_text: String,
    pub weight: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_global_flow_log_checkpoint_view(item: &GlobalFlowLogCheckpoint) -> GlobalFlowLogCheckpointView {
    GlobalFlowLogCheckpointView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        rotation_days: item.rotation_days.to_string(),
        pinned: format!("{}", item.pinned),
        floor_value: format!("{:.2}", item.floor_value),
        batch_size: item.batch_size.to_string(),
        key_id_text: text::truncate(&item.key_id, 40),
        weight: format!("{:.2}", item.weight),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_flow_log_checkpoint_views(items: &[GlobalFlowLogCheckpoint]) -> Vec<GlobalFlowLogCheckpointView> {
    let mut views: Vec<GlobalFlowLogCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_flow_log_checkpoint_view(item));
    }
    views
}

pub fn sum_global_flow_log_checkpoint_view(view: &GlobalFlowLogCheckpointView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_floor_value + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_flow_log_checkpoint("code-795", "label-248");
        let view = to_global_flow_log_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_flow_log_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
