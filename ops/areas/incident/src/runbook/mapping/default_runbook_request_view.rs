use super::*;

use acme_kit::text;

/// How screens and exports show a default runbook request.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookRequestView {
    pub code: String,
    pub name_text: String,
    pub expires_at: String,
    pub namespace_text: String,
    pub description: String,
    pub attempts: String,
    pub amount: String,
    pub score_text: String,
    pub retries: String,
    pub ceiling: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_default_runbook_request_view(item: &DefaultRunbookRequest) -> DefaultRunbookRequestView {
    DefaultRunbookRequestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        namespace_text: item.namespace.clone(),
        description: text::truncate(&item.description, 64),
        attempts: item.attempts.to_string(),
        amount: format!("{:.2}", item.amount),
        score_text: format!("{:.2}", item.score),
        retries: item.retries.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_runbook_request_views(items: &[DefaultRunbookRequest]) -> Vec<DefaultRunbookRequestView> {
    let mut views: Vec<DefaultRunbookRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_runbook_request_view(item));
    }
    views
}

pub fn sum_default_runbook_request_view(view: &DefaultRunbookRequestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_attempts as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_runbook_request("code-655", "name-372");
        let view = to_default_runbook_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_runbook_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
