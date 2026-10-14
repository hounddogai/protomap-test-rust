use super::*;

use acme_kit::text;

/// How screens and exports show a capped probe agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeAgreementView {
    pub code: String,
    pub label_text: String,
    pub approved_text: String,
    pub external_ref: String,
    pub key_id_text: String,
    pub created_at: String,
    pub segment: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_capped_probe_agreement_view(item: &CappedProbeAgreement) -> CappedProbeAgreementView {
    CappedProbeAgreementView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        approved_text: format!("{}", item.approved),
        external_ref: text::truncate(&item.external_ref, 120),
        key_id_text: item.key_id.clone(),
        created_at: item.created_at.to_string(),
        segment: item.segment.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_probe_agreement_views(items: &[CappedProbeAgreement]) -> Vec<CappedProbeAgreementView> {
    let mut views: Vec<CappedProbeAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_probe_agreement_view(item));
    }
    views
}

pub fn sum_capped_probe_agreement_view(view: &CappedProbeAgreementView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_probe_agreement("code-329", "label-414");
        let view = to_capped_probe_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_probe_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
