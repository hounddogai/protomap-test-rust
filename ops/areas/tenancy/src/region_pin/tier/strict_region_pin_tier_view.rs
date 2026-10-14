use super::*;

use acme_kit::text;

/// How screens and exports show a strict region pin tier.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinTierView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub fingerprint: String,
    pub name: String,
    pub weight: String,
    pub reviewer_text: String,
    pub approved: String,
    pub ttl_seconds: String,
    pub version: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_strict_region_pin_tier_view(item: &StrictRegionPinTier) -> StrictRegionPinTierView {
    StrictRegionPinTierView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        summary: item.summary.clone(),
        fingerprint: item.fingerprint.clone(),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        reviewer_text: item.reviewer.clone(),
        approved: format!("{}", item.approved),
        ttl_seconds: item.ttl_seconds.to_string(),
        version: item.version.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_region_pin_tier_views(items: &[StrictRegionPinTier]) -> Vec<StrictRegionPinTierView> {
    let mut views: Vec<StrictRegionPinTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_region_pin_tier_view(item));
    }
    views
}

pub fn sum_strict_region_pin_tier_view(view: &StrictRegionPinTierView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_weight + (parsed_ttl_seconds as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_region_pin_tier("code-504", "title-901");
        let view = to_strict_region_pin_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_region_pin_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
