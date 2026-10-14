use super::*;

/// How screens and exports show a evidence agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAgreementView {
    pub code: String,
    pub label: String,
    pub rotation_days_text: String,
    pub discount: String,
    pub version: String,
    pub principal: String,
    pub tenant_id: String,
    pub segment_text: String,
    pub scope: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_evidence_agreement_view(item: &EvidenceAgreement) -> EvidenceAgreementView {
    EvidenceAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        discount: format!("{:.2}", item.discount),
        version: item.version.to_string(),
        principal: item.principal.clone(),
        tenant_id: item.tenant_id.clone(),
        segment_text: item.segment.clone(),
        scope: item.scope.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_agreement_views(items: &[EvidenceAgreement]) -> Vec<EvidenceAgreementView> {
    let mut views: Vec<EvidenceAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_agreement_view(item));
    }
    views
}

pub fn sum_evidence_agreement_view(view: &EvidenceAgreementView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_discount + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_agreement("code-984", "label-40");
        let view = to_evidence_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
