use super::*;

use acme_kit::text;

/// How screens and exports show a tiered egress digest.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressDigestView {
    pub code: String,
    pub name: String,
    pub namespace_text: String,
    pub tenant_id: String,
    pub amount: String,
    pub expires_at: String,
    pub capacity: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_tiered_egress_digest_view(item: &TieredEgressDigest) -> TieredEgressDigestView {
    TieredEgressDigestView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        namespace_text: item.namespace.clone(),
        tenant_id: item.tenant_id.clone(),
        amount: format!("{:.2}", item.amount),
        expires_at: item.expires_at.to_string(),
        capacity: item.capacity.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_egress_digest_views(items: &[TieredEgressDigest]) -> Vec<TieredEgressDigestView> {
    let mut views: Vec<TieredEgressDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_egress_digest_view(item));
    }
    views
}

pub fn sum_tiered_egress_digest_view(view: &TieredEgressDigestView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_amount + (parsed_expires_at as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_egress_digest("code-264", "name-311");
        let view = to_tiered_egress_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_egress_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
