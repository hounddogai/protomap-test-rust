use super::*;

use acme_kit::text;

/// How screens and exports show a weighted region pin review.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinReviewView {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub amount: String,
    pub timezone_text: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_weighted_region_pin_review_view(item: &WeightedRegionPinReview) -> WeightedRegionPinReviewView {
    WeightedRegionPinReviewView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        notes: text::truncate(&item.notes, 80),
        amount: format!("{:.2}", item.amount),
        timezone_text: item.timezone.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_region_pin_review_views(items: &[WeightedRegionPinReview]) -> Vec<WeightedRegionPinReviewView> {
    let mut views: Vec<WeightedRegionPinReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_region_pin_review_view(item));
    }
    views
}

pub fn sum_weighted_region_pin_review_view(view: &WeightedRegionPinReviewView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_region_pin_review("code-834", "label-639");
        let view = to_weighted_region_pin_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_region_pin_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
