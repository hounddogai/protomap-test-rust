use super::*;

use acme_kit::text;

/// How screens and exports show a global nonce estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalNonceEstimateView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub priority: String,
    pub limit: String,
    pub contact_email_text: String,
    pub external_ref_text: String,
    pub principal: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_global_nonce_estimate_view(item: &GlobalNonceEstimate) -> GlobalNonceEstimateView {
    GlobalNonceEstimateView {
        code: item.code.clone(),
        name: item.name.clone(),
        attempts: item.attempts.to_string(),
        priority: item.priority.to_string(),
        limit: item.limit.to_string(),
        contact_email_text: item.contact_email.clone(),
        external_ref_text: item.external_ref.clone(),
        principal: text::truncate(&item.principal, 64),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_nonce_estimate_views(items: &[GlobalNonceEstimate]) -> Vec<GlobalNonceEstimateView> {
    let mut views: Vec<GlobalNonceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_nonce_estimate_view(item));
    }
    views
}

pub fn sum_global_nonce_estimate_view(view: &GlobalNonceEstimateView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_priority as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_nonce_estimate("code-534", "name-334");
        let view = to_global_nonce_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_nonce_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
