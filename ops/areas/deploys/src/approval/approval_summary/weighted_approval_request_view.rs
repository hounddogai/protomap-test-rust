use super::*;

use acme_kit::text;

/// How screens and exports show a weighted approval request.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedApprovalRequestView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub rotation_days_text: String,
    pub window_days: String,
    pub locale_text: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_weighted_approval_request_view(item: &WeightedApprovalRequest) -> WeightedApprovalRequestView {
    WeightedApprovalRequestView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 80),
        sequence: item.sequence.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        window_days: item.window_days.to_string(),
        locale_text: text::truncate(&item.locale, 80),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_approval_request_views(items: &[WeightedApprovalRequest]) -> Vec<WeightedApprovalRequestView> {
    let mut views: Vec<WeightedApprovalRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_approval_request_view(item));
    }
    views
}

pub fn sum_weighted_approval_request_view(view: &WeightedApprovalRequestView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_approval_request("code-755", "name-63");
        let view = to_weighted_approval_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_approval_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
