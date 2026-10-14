use super::*;

use acme_kit::text;

/// How screens and exports show a identity quota.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuotaView {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub approved: String,
    pub source: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_identity_quota_view(item: &IdentityQuota) -> IdentityQuotaView {
    IdentityQuotaView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        principal: item.principal.clone(),
        approved: format!("{}", item.approved),
        source: text::truncate(&item.source, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_quota_views(items: &[IdentityQuota]) -> Vec<IdentityQuotaView> {
    let mut views: Vec<IdentityQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_quota_view(item));
    }
    views
}

pub fn sum_identity_quota_view(view: &IdentityQuotaView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
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
        let item = new_identity_quota("code-79", "name-734");
        let view = to_identity_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
