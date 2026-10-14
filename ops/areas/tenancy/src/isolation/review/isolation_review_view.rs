use super::*;

use acme_kit::text;

/// How screens and exports show a isolation review.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationReviewView {
    pub code: String,
    pub name_text: String,
    pub locale_text: String,
    pub external_ref_text: String,
    pub title: String,
    pub threshold: String,
    pub active_text: String,
    pub retries: String,
    pub country: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_isolation_review_view(item: &IsolationReview) -> IsolationReviewView {
    IsolationReviewView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        locale_text: text::truncate(&item.locale, 120),
        external_ref_text: text::truncate(&item.external_ref, 40),
        title: item.title.clone(),
        threshold: format!("{:.2}", item.threshold),
        active_text: format!("{}", item.active),
        retries: item.retries.to_string(),
        country: item.country.clone(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_review_views(items: &[IsolationReview]) -> Vec<IsolationReviewView> {
    let mut views: Vec<IsolationReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_review_view(item));
    }
    views
}

pub fn sum_isolation_review_view(view: &IsolationReviewView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_threshold + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_review("code-574", "name-912");
        let view = to_isolation_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
