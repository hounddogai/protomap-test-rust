use super::*;

use acme_kit::text;

/// How screens and exports show a federation review.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationReviewView {
    pub code: String,
    pub name_text: String,
    pub label: String,
    pub duration_minutes: String,
    pub limit: String,
    pub currency_text: String,
    pub contact_email_text: String,
    pub ttl_seconds_text: String,
    pub threshold: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_federation_review_view(item: &FederationReview) -> FederationReviewView {
    FederationReviewView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 64),
        label: text::truncate(&item.label, 120),
        duration_minutes: item.duration_minutes.to_string(),
        limit: item.limit.to_string(),
        currency_text: text::truncate(&item.currency, 80),
        contact_email_text: item.contact_email.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        threshold: format!("{:.2}", item.threshold),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_review_views(items: &[FederationReview]) -> Vec<FederationReviewView> {
    let mut views: Vec<FederationReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_review_view(item));
    }
    views
}

pub fn sum_federation_review_view(view: &FederationReviewView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_limit as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_review("code-589", "name-818");
        let view = to_federation_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
