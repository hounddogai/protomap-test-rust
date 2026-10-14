use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive runbook review.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookReviewView {
    pub code: String,
    pub title_text: String,
    pub rate_text: String,
    pub timezone: String,
    pub cipher: String,
    pub source: String,
    pub amount: String,
    pub maximum: String,
    pub unit: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_adaptive_runbook_review_view(item: &AdaptiveRunbookReview) -> AdaptiveRunbookReviewView {
    AdaptiveRunbookReviewView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        rate_text: format!("{:.2}", item.rate),
        timezone: text::truncate(&item.timezone, 64),
        cipher: item.cipher.clone(),
        source: text::truncate(&item.source, 80),
        amount: format!("{:.2}", item.amount),
        maximum: format!("{:.2}", item.maximum),
        unit: item.unit.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_runbook_review_views(items: &[AdaptiveRunbookReview]) -> Vec<AdaptiveRunbookReviewView> {
    let mut views: Vec<AdaptiveRunbookReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_runbook_review_view(item));
    }
    views
}

pub fn sum_adaptive_runbook_review_view(view: &AdaptiveRunbookReviewView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + parsed_amount + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_runbook_review("code-494", "title-521");
        let view = to_adaptive_runbook_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_runbook_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
