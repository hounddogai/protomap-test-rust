use super::*;

use acme_kit::text;

/// How screens and exports show a region pin review.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReviewView {
    pub code: String,
    pub label: String,
    pub created_at: String,
    pub namespace: String,
    pub cluster: String,
    pub score: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_region_pin_review_view(item: &RegionPinReview) -> RegionPinReviewView {
    RegionPinReviewView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        namespace: item.namespace.clone(),
        cluster: item.cluster.clone(),
        score: format!("{:.2}", item.score),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_review_views(items: &[RegionPinReview]) -> Vec<RegionPinReviewView> {
    let mut views: Vec<RegionPinReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_review_view(item));
    }
    views
}

pub fn sum_region_pin_review_view(view: &RegionPinReviewView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_review("code-320", "label-909");
        let view = to_region_pin_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
