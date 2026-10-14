use super::*;

/// How screens and exports show a pending approval checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApprovalCheckpointView {
    pub code: String,
    pub title: String,
    pub threshold: String,
    pub phone_text: String,
    pub minimum: String,
    pub duration_minutes: String,
    pub floor_value_text: String,
    pub cost: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_pending_approval_checkpoint_view(item: &PendingApprovalCheckpoint) -> PendingApprovalCheckpointView {
    PendingApprovalCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        threshold: format!("{:.2}", item.threshold),
        phone_text: item.phone.clone(),
        minimum: format!("{:.2}", item.minimum),
        duration_minutes: item.duration_minutes.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        cost: format!("{:.2}", item.cost),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_approval_checkpoint_views(items: &[PendingApprovalCheckpoint]) -> Vec<PendingApprovalCheckpointView> {
    let mut views: Vec<PendingApprovalCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_approval_checkpoint_view(item));
    }
    views
}

pub fn sum_pending_approval_checkpoint_view(view: &PendingApprovalCheckpointView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_threshold + parsed_minimum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_approval_checkpoint("code-981", "title-494");
        let view = to_pending_approval_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_approval_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
