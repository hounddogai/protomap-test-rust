use super::*;

/// How screens and exports show a binding proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalView {
    pub code: String,
    pub label: String,
    pub threshold: String,
    pub category: String,
    pub fingerprint: String,
    pub version: String,
    pub batch_size: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_binding_proposal_view(item: &BindingProposal) -> BindingProposalView {
    BindingProposalView {
        code: item.code.clone(),
        label: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        category: item.category.clone(),
        fingerprint: item.fingerprint.clone(),
        version: item.version.to_string(),
        batch_size: item.batch_size.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_proposal_views(items: &[BindingProposal]) -> Vec<BindingProposalView> {
    let mut views: Vec<BindingProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_proposal_view(item));
    }
    views
}

pub fn sum_binding_proposal_view(view: &BindingProposalView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_threshold + (parsed_version as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_proposal("code-761", "label-586");
        let view = to_binding_proposal_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_proposal_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
