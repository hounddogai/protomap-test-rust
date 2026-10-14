use super::*;

use acme_kit::text;

/// How screens and exports show a primary evidence claim.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceClaimView {
    pub code_text: String,
    pub label: String,
    pub tenant_id: String,
    pub quantity: String,
    pub amount: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_primary_evidence_claim_view(item: &PrimaryEvidenceClaim) -> PrimaryEvidenceClaimView {
    PrimaryEvidenceClaimView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        tenant_id: item.tenant_id.clone(),
        quantity: item.quantity.to_string(),
        amount: format!("{:.2}", item.amount),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_evidence_claim_views(items: &[PrimaryEvidenceClaim]) -> Vec<PrimaryEvidenceClaimView> {
    let mut views: Vec<PrimaryEvidenceClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_evidence_claim_view(item));
    }
    views
}

pub fn sum_primary_evidence_claim_view(view: &PrimaryEvidenceClaimView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_amount + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_evidence_claim("code-382", "label-69");
        let view = to_primary_evidence_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_evidence_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
