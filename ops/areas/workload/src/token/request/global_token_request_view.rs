use super::*;

/// How screens and exports show a global token request.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestView {
    pub code: String,
    pub title: String,
    pub attempts: String,
    pub amount_text: String,
    pub grace_days: String,
    pub visible: String,
    pub created_at: String,
    pub ttl_seconds: String,
    pub contact_email: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_global_token_request_view(item: &GlobalTokenRequest) -> GlobalTokenRequestView {
    GlobalTokenRequestView {
        code: item.code.clone(),
        title: item.title.clone(),
        attempts: item.attempts.to_string(),
        amount_text: format!("{:.2}", item.amount),
        grace_days: item.grace_days.to_string(),
        visible: format!("{}", item.visible),
        created_at: item.created_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        contact_email: item.contact_email.clone(),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_token_request_views(items: &[GlobalTokenRequest]) -> Vec<GlobalTokenRequestView> {
    let mut views: Vec<GlobalTokenRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_token_request_view(item));
    }
    views
}

pub fn sum_global_token_request_view(view: &GlobalTokenRequestView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_amount + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_token_request("code-483", "title-231");
        let view = to_global_token_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_token_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
