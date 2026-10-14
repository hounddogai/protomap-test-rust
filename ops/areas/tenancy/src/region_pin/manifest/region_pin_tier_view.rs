use super::*;

use acme_kit::text;

/// How screens and exports show a region pin tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTierView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub maximum: String,
    pub audience: String,
    pub approved_text: String,
    pub title: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_region_pin_tier_view(item: &RegionPinTier) -> RegionPinTierView {
    RegionPinTierView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        country: text::truncate(&item.country, 80),
        maximum: format!("{:.2}", item.maximum),
        audience: item.audience.clone(),
        approved_text: format!("{}", item.approved),
        title: item.title.clone(),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_tier_views(items: &[RegionPinTier]) -> Vec<RegionPinTierView> {
    let mut views: Vec<RegionPinTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_tier_view(item));
    }
    views
}

pub fn sum_region_pin_tier_view(view: &RegionPinTierView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_tier("code-517", "label-685");
        let view = to_region_pin_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
