use super::*;

use acme_kit::text;

/// How screens and exports show a peering review.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReviewView {
    pub code: String,
    pub label_text: String,
    pub timezone: String,
    pub quota_units_text: String,
    pub threshold_text: String,
    pub batch_size_text: String,
    pub port: String,
    pub pinned: String,
    pub checksum: String,
    pub source: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_peering_review_view(item: &PeeringReview) -> PeeringReviewView {
    PeeringReviewView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        timezone: item.timezone.clone(),
        quota_units_text: item.quota_units.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        batch_size_text: item.batch_size.to_string(),
        port: item.port.to_string(),
        pinned: format!("{}", item.pinned),
        checksum: item.checksum.clone(),
        source: text::truncate(&item.source, 40),
        currency_text: text::truncate(&item.currency, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_review_views(items: &[PeeringReview]) -> Vec<PeeringReviewView> {
    let mut views: Vec<PeeringReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_review_view(item));
    }
    views
}

pub fn sum_peering_review_view(view: &PeeringReviewView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_threshold + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_review("code-356", "label-745");
        let view = to_peering_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
