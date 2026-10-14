use super::*;

use acme_kit::text;

/// How screens and exports show a zone threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThresholdView {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub owner_id: String,
    pub principal_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_zone_threshold_view(item: &ZoneThreshold) -> ZoneThresholdView {
    ZoneThresholdView {
        code: item.code.clone(),
        label: item.label.clone(),
        namespace: item.namespace.clone(),
        owner_id: text::truncate(&item.owner_id, 120),
        principal_text: text::truncate(&item.principal, 64),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_threshold_views(items: &[ZoneThreshold]) -> Vec<ZoneThresholdView> {
    let mut views: Vec<ZoneThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_threshold_view(item));
    }
    views
}

pub fn sum_zone_threshold_view(view: &ZoneThresholdView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
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
        let item = new_zone_threshold("code-148", "label-19");
        let view = to_zone_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
