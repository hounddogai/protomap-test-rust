use super::*;

use acme_kit::text;

/// How screens and exports show a legacy evidence claim.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceClaimView {
    pub code: String,
    pub title: String,
    pub version: String,
    pub grace_days: String,
    pub notes: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_legacy_evidence_claim_view(item: &LegacyEvidenceClaim) -> LegacyEvidenceClaimView {
    LegacyEvidenceClaimView {
        code: item.code.clone(),
        title: item.title.clone(),
        version: item.version.to_string(),
        grace_days: item.grace_days.to_string(),
        notes: text::truncate(&item.notes, 120),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_evidence_claim_views(items: &[LegacyEvidenceClaim]) -> Vec<LegacyEvidenceClaimView> {
    let mut views: Vec<LegacyEvidenceClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_evidence_claim_view(item));
    }
    views
}

pub fn sum_legacy_evidence_claim_view(view: &LegacyEvidenceClaimView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_version as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_evidence_claim("code-982", "title-163");
        let view = to_legacy_evidence_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_evidence_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
