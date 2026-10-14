use super::*;

use acme_kit::text;

/// How screens and exports show a pending release rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseRuleView {
    pub code: String,
    pub label: String,
    pub rate: String,
    pub contact_email_text: String,
    pub phone: String,
    pub pinned_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_pending_release_rule_view(item: &PendingReleaseRule) -> PendingReleaseRuleView {
    PendingReleaseRuleView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 64),
        rate: format!("{:.2}", item.rate),
        contact_email_text: item.contact_email.clone(),
        phone: item.phone.clone(),
        pinned_text: format!("{}", item.pinned),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_release_rule_views(items: &[PendingReleaseRule]) -> Vec<PendingReleaseRuleView> {
    let mut views: Vec<PendingReleaseRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_release_rule_view(item));
    }
    views
}

pub fn sum_pending_release_rule_view(view: &PendingReleaseRuleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_rate + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_release_rule("code-319", "label-395");
        let view = to_pending_release_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_release_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
