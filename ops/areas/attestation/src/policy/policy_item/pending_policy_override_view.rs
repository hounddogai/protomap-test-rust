use super::*;

/// How screens and exports show a pending policy override.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyOverrideView {
    pub code_text: String,
    pub name: String,
    pub minimum: String,
    pub checksum_text: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_pending_policy_override_view(item: &PendingPolicyOverride) -> PendingPolicyOverrideView {
    PendingPolicyOverrideView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        checksum_text: item.checksum.clone(),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_policy_override_views(items: &[PendingPolicyOverride]) -> Vec<PendingPolicyOverrideView> {
    let mut views: Vec<PendingPolicyOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_policy_override_view(item));
    }
    views
}

pub fn sum_pending_policy_override_view(view: &PendingPolicyOverrideView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_policy_override("code-474", "name-873");
        let view = to_pending_policy_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_policy_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
