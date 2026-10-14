use super::*;

use acme_kit::text;

/// How screens and exports show a evidence claim.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceClaimView {
    pub code: String,
    pub name: String,
    pub ceiling_text: String,
    pub cipher_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_evidence_claim_view(item: &EvidenceClaim) -> EvidenceClaimView {
    EvidenceClaimView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        cipher_text: item.cipher.clone(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_claim_views(items: &[EvidenceClaim]) -> Vec<EvidenceClaimView> {
    let mut views: Vec<EvidenceClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_claim_view(item));
    }
    views
}

pub fn sum_evidence_claim_view(view: &EvidenceClaimView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_claim("code-796", "name-122");
        let view = to_evidence_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
