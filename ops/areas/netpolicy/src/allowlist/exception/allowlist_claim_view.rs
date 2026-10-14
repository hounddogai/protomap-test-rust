use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist claim.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistClaimView {
    pub code: String,
    pub label: String,
    pub external_ref_text: String,
    pub capacity_text: String,
    pub amount: String,
    pub ceiling: String,
    pub cluster_text: String,
    pub region: String,
    pub notes: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_allowlist_claim_view(item: &AllowlistClaim) -> AllowlistClaimView {
    AllowlistClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        external_ref_text: item.external_ref.clone(),
        capacity_text: item.capacity.to_string(),
        amount: format!("{:.2}", item.amount),
        ceiling: format!("{:.2}", item.ceiling),
        cluster_text: item.cluster.clone(),
        region: text::truncate(&item.region, 64),
        notes: item.notes.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_claim_views(items: &[AllowlistClaim]) -> Vec<AllowlistClaimView> {
    let mut views: Vec<AllowlistClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_claim_view(item));
    }
    views
}

pub fn sum_allowlist_claim_view(view: &AllowlistClaimView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_amount + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_claim("code-877", "label-330");
        let view = to_allowlist_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
