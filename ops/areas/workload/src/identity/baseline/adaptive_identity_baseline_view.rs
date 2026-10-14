use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive identity baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselineView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub pinned: String,
    pub batch_size: String,
    pub weight_text: String,
    pub ttl_seconds: String,
    pub key_id_text: String,
    pub grace_days: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_adaptive_identity_baseline_view(item: &AdaptiveIdentityBaseline) -> AdaptiveIdentityBaselineView {
    AdaptiveIdentityBaselineView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        owner_id: item.owner_id.clone(),
        pinned: format!("{}", item.pinned),
        batch_size: item.batch_size.to_string(),
        weight_text: format!("{:.2}", item.weight),
        ttl_seconds: item.ttl_seconds.to_string(),
        key_id_text: item.key_id.clone(),
        grace_days: item.grace_days.to_string(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_identity_baseline_views(items: &[AdaptiveIdentityBaseline]) -> Vec<AdaptiveIdentityBaselineView> {
    let mut views: Vec<AdaptiveIdentityBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_identity_baseline_view(item));
    }
    views
}

pub fn sum_adaptive_identity_baseline_view(view: &AdaptiveIdentityBaselineView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_weight + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_identity_baseline("code-99", "name-578");
        let view = to_adaptive_identity_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_identity_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
