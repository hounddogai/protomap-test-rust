use super::*;

/// How screens and exports show a legacy policy exception.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyExceptionView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub sequence: String,
    pub quantity_text: String,
    pub audience_text: String,
    pub retries: String,
    pub ttl_seconds: String,
    pub owner_id: String,
    pub rotation_days: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_legacy_policy_exception_view(item: &LegacyPolicyException) -> LegacyPolicyExceptionView {
    LegacyPolicyExceptionView {
        code: item.code.clone(),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        sequence: item.sequence.to_string(),
        quantity_text: item.quantity.to_string(),
        audience_text: item.audience.clone(),
        retries: item.retries.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        owner_id: item.owner_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_policy_exception_views(items: &[LegacyPolicyException]) -> Vec<LegacyPolicyExceptionView> {
    let mut views: Vec<LegacyPolicyExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_policy_exception_view(item));
    }
    views
}

pub fn sum_legacy_policy_exception_view(view: &LegacyPolicyExceptionView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_amount + (parsed_sequence as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_policy_exception("code-791", "label-762");
        let view = to_legacy_policy_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_policy_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
