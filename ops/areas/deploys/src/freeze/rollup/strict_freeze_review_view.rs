use super::*;

use acme_kit::text;

/// How screens and exports show a strict freeze review.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub margin: String,
    pub pinned_text: String,
    pub archived_text: String,
    pub rate: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_strict_freeze_review_view(item: &StrictFreezeReview) -> StrictFreezeReviewView {
    StrictFreezeReviewView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        attempts: item.attempts.to_string(),
        margin: format!("{:.2}", item.margin),
        pinned_text: format!("{}", item.pinned),
        archived_text: format!("{}", item.archived),
        rate: format!("{:.2}", item.rate),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_freeze_review_views(items: &[StrictFreezeReview]) -> Vec<StrictFreezeReviewView> {
    let mut views: Vec<StrictFreezeReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_freeze_review_view(item));
    }
    views
}

pub fn sum_strict_freeze_review_view(view: &StrictFreezeReviewView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_margin + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_freeze_review("code-642", "name-122");
        let view = to_strict_freeze_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_freeze_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
