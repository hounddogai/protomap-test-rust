use super::*;

/// How screens and exports show a federation tier.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationTierView {
    pub code: String,
    pub name: String,
    pub created_at: String,
    pub locale: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_federation_tier_view(item: &FederationTier) -> FederationTierView {
    FederationTierView {
        code: item.code.clone(),
        name: item.name.clone(),
        created_at: item.created_at.to_string(),
        locale: item.locale.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_tier_views(items: &[FederationTier]) -> Vec<FederationTierView> {
    let mut views: Vec<FederationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_tier_view(item));
    }
    views
}

pub fn sum_federation_tier_view(view: &FederationTierView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_starts_at as f64))
}
