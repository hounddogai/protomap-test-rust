use super::*;

use acme_kit::text;

/// How screens and exports show a quote audit.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAuditView {
    pub code: String,
    pub label_text: String,
    pub channel: String,
    pub budget: String,
    pub burst_text: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_quote_audit_view(item: &QuoteAudit) -> QuoteAuditView {
    QuoteAuditView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        channel: text::truncate(&item.channel, 80),
        budget: format!("{:.2}", item.budget),
        burst_text: item.burst.to_string(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_audit_views(items: &[QuoteAudit]) -> Vec<QuoteAuditView> {
    let mut views: Vec<QuoteAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_audit_view(item));
    }
    views
}

pub fn sum_quote_audit_view(view: &QuoteAuditView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_budget + (parsed_burst as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_audit("code-527", "label-363");
        let view = to_quote_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
