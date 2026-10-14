use super::*;

use acme_kit::text;

/// How screens and exports show a pending probe draft.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingProbeDraftView {
    pub code: String,
    pub label: String,
    pub tenant_id_text: String,
    pub window_days_text: String,
    pub owner_id: String,
    pub reviewer: String,
    pub rotation_days: String,
    pub floor_value: String,
    pub burst: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_pending_probe_draft_view(item: &PendingProbeDraft) -> PendingProbeDraftView {
    PendingProbeDraftView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 64),
        window_days_text: item.window_days.to_string(),
        owner_id: text::truncate(&item.owner_id, 80),
        reviewer: item.reviewer.clone(),
        rotation_days: item.rotation_days.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        burst: item.burst.to_string(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_probe_draft_views(items: &[PendingProbeDraft]) -> Vec<PendingProbeDraftView> {
    let mut views: Vec<PendingProbeDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_probe_draft_view(item));
    }
    views
}

pub fn sum_pending_probe_draft_view(view: &PendingProbeDraftView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
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
    Ok(parsed_window_days as f64 + (parsed_rotation_days as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_probe_draft("code-770", "label-729");
        let view = to_pending_probe_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_probe_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
