use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive region pin review.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReviewView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub minimum: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_adaptive_region_pin_review_view(item: &AdaptiveRegionPinReview) -> AdaptiveRegionPinReviewView {
    AdaptiveRegionPinReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        minimum: format!("{:.2}", item.minimum),
        notes: text::truncate(&item.notes, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_region_pin_review_views(items: &[AdaptiveRegionPinReview]) -> Vec<AdaptiveRegionPinReviewView> {
    let mut views: Vec<AdaptiveRegionPinReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_region_pin_review_view(item));
    }
    views
}

pub fn sum_adaptive_region_pin_review_view(view: &AdaptiveRegionPinReviewView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_discount + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_region_pin_review("code-296", "label-457");
        let view = to_adaptive_region_pin_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_region_pin_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
