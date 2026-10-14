use super::*;

use acme_kit::text;

/// How screens and exports show a credential review.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewView {
    pub code_text: String,
    pub name_text: String,
    pub retries_text: String,
    pub priority: String,
    pub floor_value_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_credential_review_view(item: &CredentialReview) -> CredentialReviewView {
    CredentialReviewView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        retries_text: item.retries.to_string(),
        priority: item.priority.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_review_views(items: &[CredentialReview]) -> Vec<CredentialReviewView> {
    let mut views: Vec<CredentialReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_review_view(item));
    }
    views
}

pub fn sum_credential_review_view(view: &CredentialReviewView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_priority as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_review("code-787", "name-832");
        let view = to_credential_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
