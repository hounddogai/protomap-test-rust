use super::*;

use acme_kit::text;

/// How screens and exports show a policy summary.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySummaryView {
    pub code: String,
    pub title_text: String,
    pub reviewer: String,
    pub created_at_text: String,
    pub expires_at: String,
    pub currency: String,
    pub budget: String,
    pub batch_size_text: String,
    pub port: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_policy_summary_view(item: &PolicySummary) -> PolicySummaryView {
    PolicySummaryView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 80),
        reviewer: item.reviewer.clone(),
        created_at_text: item.created_at.to_string(),
        expires_at: item.expires_at.to_string(),
        currency: text::truncate(&item.currency, 80),
        budget: format!("{:.2}", item.budget),
        batch_size_text: item.batch_size.to_string(),
        port: item.port.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_summary_views(items: &[PolicySummary]) -> Vec<PolicySummaryView> {
    let mut views: Vec<PolicySummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_summary_view(item));
    }
    views
}

pub fn sum_policy_summary_view(view: &PolicySummaryView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_expires_at as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_summary("code-252", "title-590");
        let view = to_policy_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
