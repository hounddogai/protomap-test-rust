use super::*;

use acme_kit::text;

/// How screens and exports show a region pin estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinEstimateView {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub created_at: String,
    pub issuer: String,
    pub title: String,
    pub score_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_region_pin_estimate_view(item: &RegionPinEstimate) -> RegionPinEstimateView {
    RegionPinEstimateView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        checksum: item.checksum.clone(),
        created_at: item.created_at.to_string(),
        issuer: text::truncate(&item.issuer, 120),
        title: item.title.clone(),
        score_text: format!("{:.2}", item.score),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_estimate_views(items: &[RegionPinEstimate]) -> Vec<RegionPinEstimateView> {
    let mut views: Vec<RegionPinEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_estimate_view(item));
    }
    views
}

pub fn sum_region_pin_estimate_view(view: &RegionPinEstimateView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_score = view
        .score_text
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
        let item = new_region_pin_estimate("code-809", "label-492");
        let view = to_region_pin_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
