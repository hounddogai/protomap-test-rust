use super::*;

use acme_kit::text;

/// How screens and exports show a legacy federation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAdjustmentView {
    pub code: String,
    pub name_text: String,
    pub expires_at_text: String,
    pub ceiling_text: String,
    pub ends_at: String,
    pub phone_text: String,
    pub segment_text: String,
    pub grace_days_text: String,
    pub limit_text: String,
    pub retries: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_legacy_federation_adjustment_view(item: &LegacyFederationAdjustment) -> LegacyFederationAdjustmentView {
    LegacyFederationAdjustmentView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        expires_at_text: item.expires_at.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        ends_at: item.ends_at.to_string(),
        phone_text: item.phone.clone(),
        segment_text: item.segment.clone(),
        grace_days_text: item.grace_days.to_string(),
        limit_text: item.limit.to_string(),
        retries: item.retries.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_federation_adjustment_views(
    items: &[LegacyFederationAdjustment],
) -> Vec<LegacyFederationAdjustmentView> {
    let mut views: Vec<LegacyFederationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_federation_adjustment_view(item));
    }
    views
}

pub fn sum_legacy_federation_adjustment_view(view: &LegacyFederationAdjustmentView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ceiling + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_federation_adjustment("code-138", "name-330");
        let view = to_legacy_federation_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_federation_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
