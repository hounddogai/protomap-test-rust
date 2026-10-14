use super::*;

use acme_kit::text;

/// How screens and exports show a legacy evidence estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceEstimateView {
    pub code_text: String,
    pub name_text: String,
    pub host: String,
    pub locale: String,
    pub minimum: String,
    pub cluster: String,
    pub limit: String,
    pub version_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_legacy_evidence_estimate_view(item: &LegacyEvidenceEstimate) -> LegacyEvidenceEstimateView {
    LegacyEvidenceEstimateView {
        code_text: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 64),
        host: item.host.clone(),
        locale: item.locale.clone(),
        minimum: format!("{:.2}", item.minimum),
        cluster: item.cluster.clone(),
        limit: item.limit.to_string(),
        version_text: item.version.to_string(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_evidence_estimate_views(items: &[LegacyEvidenceEstimate]) -> Vec<LegacyEvidenceEstimateView> {
    let mut views: Vec<LegacyEvidenceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_evidence_estimate_view(item));
    }
    views
}

pub fn sum_legacy_evidence_estimate_view(view: &LegacyEvidenceEstimateView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_minimum + (parsed_limit as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_evidence_estimate("code-787", "name-468");
        let view = to_legacy_evidence_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_evidence_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
