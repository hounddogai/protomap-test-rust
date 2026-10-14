use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled quote review.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteReviewView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub ends_at: String,
    pub fingerprint_text: String,
    pub version_text: String,
    pub duration_minutes_text: String,
    pub external_ref: String,
    pub source: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_scheduled_quote_review_view(item: &ScheduledQuoteReview) -> ScheduledQuoteReviewView {
    ScheduledQuoteReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        country: text::truncate(&item.country, 120),
        ends_at: item.ends_at.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        version_text: item.version.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        external_ref: item.external_ref.clone(),
        source: text::truncate(&item.source, 80),
        region: text::truncate(&item.region, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_quote_review_views(items: &[ScheduledQuoteReview]) -> Vec<ScheduledQuoteReviewView> {
    let mut views: Vec<ScheduledQuoteReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_quote_review_view(item));
    }
    views
}

pub fn sum_scheduled_quote_review_view(view: &ScheduledQuoteReviewView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_version as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_quote_review("code-605", "label-588");
        let view = to_scheduled_quote_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_quote_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
