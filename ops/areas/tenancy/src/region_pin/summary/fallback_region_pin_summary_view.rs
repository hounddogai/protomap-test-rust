use super::*;

use acme_kit::text;

/// How screens and exports show a fallback region pin summary.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinSummaryView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub weight: String,
    pub expires_at: String,
    pub ttl_seconds: String,
    pub country_text: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_fallback_region_pin_summary_view(item: &FallbackRegionPinSummary) -> FallbackRegionPinSummaryView {
    FallbackRegionPinSummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        weight: format!("{:.2}", item.weight),
        expires_at: item.expires_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        country_text: text::truncate(&item.country, 64),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_region_pin_summary_views(items: &[FallbackRegionPinSummary]) -> Vec<FallbackRegionPinSummaryView> {
    let mut views: Vec<FallbackRegionPinSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_region_pin_summary_view(item));
    }
    views
}

pub fn sum_fallback_region_pin_summary_view(view: &FallbackRegionPinSummaryView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_weight + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_region_pin_summary("code-301", "title-659");
        let view = to_fallback_region_pin_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_region_pin_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
