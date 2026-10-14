use super::*;

use acme_kit::text;

/// How screens and exports show a regional probe claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeClaimView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub duration_minutes: String,
    pub notes: String,
    pub principal_text: String,
    pub expires_at: String,
    pub visible_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_regional_probe_claim_view(item: &RegionalProbeClaim) -> RegionalProbeClaimView {
    RegionalProbeClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        summary: item.summary.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        notes: text::truncate(&item.notes, 80),
        principal_text: text::truncate(&item.principal, 64),
        expires_at: item.expires_at.to_string(),
        visible_text: format!("{}", item.visible),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_probe_claim_views(items: &[RegionalProbeClaim]) -> Vec<RegionalProbeClaimView> {
    let mut views: Vec<RegionalProbeClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_probe_claim_view(item));
    }
    views
}

pub fn sum_regional_probe_claim_view(view: &RegionalProbeClaimView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_probe_claim("code-497", "label-46");
        let view = to_regional_probe_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_probe_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
