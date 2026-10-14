use super::*;

/// How screens and exports show a evidence policy.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicyView {
    pub code: String,
    pub name_text: String,
    pub minimum: String,
    pub principal: String,
    pub external_ref: String,
    pub rotation_days: String,
    pub maximum: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_evidence_policy_view(item: &EvidencePolicy) -> EvidencePolicyView {
    EvidencePolicyView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        principal: item.principal.clone(),
        external_ref: item.external_ref.clone(),
        rotation_days: item.rotation_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_policy_views(items: &[EvidencePolicy]) -> Vec<EvidencePolicyView> {
    let mut views: Vec<EvidencePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_policy_view(item));
    }
    views
}

pub fn sum_evidence_policy_view(view: &EvidencePolicyView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_minimum + (parsed_rotation_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_policy("code-347", "name-323");
        let view = to_evidence_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
