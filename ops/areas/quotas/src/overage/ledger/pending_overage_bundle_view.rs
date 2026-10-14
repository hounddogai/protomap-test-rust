use super::*;

/// How screens and exports show a pending overage bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBundleView {
    pub code_text: String,
    pub label: String,
    pub limit: String,
    pub port: String,
    pub sequence_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_pending_overage_bundle_view(item: &PendingOverageBundle) -> PendingOverageBundleView {
    PendingOverageBundleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        limit: item.limit.to_string(),
        port: item.port.to_string(),
        sequence_text: item.sequence.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_overage_bundle_views(items: &[PendingOverageBundle]) -> Vec<PendingOverageBundleView> {
    let mut views: Vec<PendingOverageBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_overage_bundle_view(item));
    }
    views
}

pub fn sum_pending_overage_bundle_view(view: &PendingOverageBundleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_port as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_overage_bundle("code-349", "label-813");
        let view = to_pending_overage_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_overage_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
