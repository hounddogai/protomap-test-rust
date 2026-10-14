use super::*;

use acme_kit::text;

/// How screens and exports show a strict rollout segment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutSegmentView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub quantity_text: String,
    pub port_text: String,
    pub title_text: String,
    pub rotation_days: String,
    pub sequence: String,
    pub attempts_text: String,
    pub budget: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_strict_rollout_segment_view(item: &StrictRolloutSegment) -> StrictRolloutSegmentView {
    StrictRolloutSegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        notes: text::truncate(&item.notes, 120),
        quantity_text: item.quantity.to_string(),
        port_text: item.port.to_string(),
        title_text: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        sequence: item.sequence.to_string(),
        attempts_text: item.attempts.to_string(),
        budget: format!("{:.2}", item.budget),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_rollout_segment_views(items: &[StrictRolloutSegment]) -> Vec<StrictRolloutSegmentView> {
    let mut views: Vec<StrictRolloutSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_rollout_segment_view(item));
    }
    views
}

pub fn sum_strict_rollout_segment_view(view: &StrictRolloutSegmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_port as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_rollout_segment("code-893", "name-743");
        let view = to_strict_rollout_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_rollout_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
