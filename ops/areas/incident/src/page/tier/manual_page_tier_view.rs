use super::*;

use acme_kit::text;

/// How screens and exports show a manual page tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageTierView {
    pub code: String,
    pub name: String,
    pub external_ref_text: String,
    pub key_id_text: String,
    pub floor_value: String,
    pub ttl_seconds: String,
    pub principal_text: String,
    pub ceiling: String,
    pub host: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_manual_page_tier_view(item: &ManualPageTier) -> ManualPageTierView {
    ManualPageTierView {
        code: item.code.clone(),
        name: item.name.clone(),
        external_ref_text: item.external_ref.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        floor_value: format!("{:.2}", item.floor_value),
        ttl_seconds: item.ttl_seconds.to_string(),
        principal_text: text::truncate(&item.principal, 80),
        ceiling: format!("{:.2}", item.ceiling),
        host: text::truncate(&item.host, 64),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_page_tier_views(items: &[ManualPageTier]) -> Vec<ManualPageTierView> {
    let mut views: Vec<ManualPageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_page_tier_view(item));
    }
    views
}

pub fn sum_manual_page_tier_view(view: &ManualPageTierView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_floor_value + (parsed_ttl_seconds as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_page_tier("code-227", "name-220");
        let view = to_manual_page_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_page_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
