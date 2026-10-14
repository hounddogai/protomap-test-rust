use super::*;

use acme_kit::text;

/// How screens and exports show a tiered delegation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaimView {
    pub code: String,
    pub label: String,
    pub external_ref_text: String,
    pub rotation_days: String,
    pub cost: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_tiered_delegation_claim_view(item: &TieredDelegationClaim) -> TieredDelegationClaimView {
    TieredDelegationClaimView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        external_ref_text: item.external_ref.clone(),
        rotation_days: item.rotation_days.to_string(),
        cost: format!("{:.2}", item.cost),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_delegation_claim_views(items: &[TieredDelegationClaim]) -> Vec<TieredDelegationClaimView> {
    let mut views: Vec<TieredDelegationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_delegation_claim_view(item));
    }
    views
}

pub fn sum_tiered_delegation_claim_view(view: &TieredDelegationClaimView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_cost + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_delegation_claim("code-490", "label-325");
        let view = to_tiered_delegation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_delegation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
