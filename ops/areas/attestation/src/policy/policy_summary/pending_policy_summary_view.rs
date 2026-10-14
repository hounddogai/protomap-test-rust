use super::*;

use acme_kit::text;

/// How screens and exports show a pending policy summary.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicySummaryView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub contact_email_text: String,
    pub rotation_days_text: String,
    pub archived: String,
    pub quota_units: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_pending_policy_summary_view(item: &PendingPolicySummary) -> PendingPolicySummaryView {
    PendingPolicySummaryView {
        code: item.code.clone(),
        name: item.name.clone(),
        summary: text::truncate(&item.summary, 64),
        contact_email_text: item.contact_email.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        archived: format!("{}", item.archived),
        quota_units: item.quota_units.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_policy_summary_views(items: &[PendingPolicySummary]) -> Vec<PendingPolicySummaryView> {
    let mut views: Vec<PendingPolicySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_policy_summary_view(item));
    }
    views
}

pub fn sum_pending_policy_summary_view(view: &PendingPolicySummaryView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_quota_units as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_policy_summary("code-864", "name-390");
        let view = to_pending_policy_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_policy_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
