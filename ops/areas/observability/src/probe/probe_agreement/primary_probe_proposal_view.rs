use super::*;

/// How screens and exports show a primary probe proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryProbeProposalView {
    pub code_text: String,
    pub label: String,
    pub priority: String,
    pub country: String,
    pub region_text: String,
    pub minimum: String,
    pub expires_at: String,
    pub window_days: String,
    pub fingerprint: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_primary_probe_proposal_view(item: &PrimaryProbeProposal) -> PrimaryProbeProposalView {
    PrimaryProbeProposalView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        country: item.country.clone(),
        region_text: item.region.clone(),
        minimum: format!("{:.2}", item.minimum),
        expires_at: item.expires_at.to_string(),
        window_days: item.window_days.to_string(),
        fingerprint: item.fingerprint.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_probe_proposal_views(items: &[PrimaryProbeProposal]) -> Vec<PrimaryProbeProposalView> {
    let mut views: Vec<PrimaryProbeProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_probe_proposal_view(item));
    }
    views
}

pub fn sum_primary_probe_proposal_view(view: &PrimaryProbeProposalView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_priority as f64 + parsed_minimum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_probe_proposal("code-901", "label-200");
        let view = to_primary_probe_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_probe_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
