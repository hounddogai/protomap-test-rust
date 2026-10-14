use super::*;

use acme_kit::text;

/// How screens and exports show a timeline review.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReviewView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub currency: String,
    pub channel: String,
    pub visible: String,
    pub unit_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_timeline_review_view(item: &TimelineReview) -> TimelineReviewView {
    TimelineReviewView {
        code: item.code.clone(),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        currency: text::truncate(&item.currency, 120),
        channel: text::truncate(&item.channel, 120),
        visible: format!("{}", item.visible),
        unit_text: text::truncate(&item.unit, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_review_views(items: &[TimelineReview]) -> Vec<TimelineReviewView> {
    let mut views: Vec<TimelineReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_review_view(item));
    }
    views
}

pub fn sum_timeline_review_view(view: &TimelineReviewView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_review("code-987", "title-227");
        let view = to_timeline_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
