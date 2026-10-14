use super::*;

use acme_kit::text;

/// How screens and exports show a slo tier.
#[derive(Debug, Clone, PartialEq)]
pub struct SloTierView {
    pub code_text: String,
    pub label_text: String,
    pub contact_email: String,
    pub region_text: String,
    pub limit_text: String,
    pub cluster_text: String,
    pub scope_text: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_slo_tier_view(item: &SloTier) -> SloTierView {
    SloTierView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        contact_email: text::truncate(&item.contact_email, 64),
        region_text: item.region.clone(),
        limit_text: item.limit.to_string(),
        cluster_text: item.cluster.clone(),
        scope_text: item.scope.clone(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_tier_views(items: &[SloTier]) -> Vec<SloTierView> {
    let mut views: Vec<SloTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_tier_view(item));
    }
    views
}

pub fn sum_slo_tier_view(view: &SloTierView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_tier("code-768", "label-612");
        let view = to_slo_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
