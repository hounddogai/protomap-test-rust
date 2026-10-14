use super::*;

/// How screens and exports show a rule claim.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimView {
    pub code: String,
    pub label_text: String,
    pub country: String,
    pub threshold_text: String,
    pub grace_days: String,
    pub ceiling: String,
    pub phone: String,
    pub retries: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_rule_claim_view(item: &RuleClaim) -> RuleClaimView {
    RuleClaimView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        country: item.country.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        grace_days: item.grace_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        phone: item.phone.clone(),
        retries: item.retries.to_string(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_claim_views(items: &[RuleClaim]) -> Vec<RuleClaimView> {
    let mut views: Vec<RuleClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_claim_view(item));
    }
    views
}

pub fn sum_rule_claim_view(view: &RuleClaimView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_threshold + (parsed_grace_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_claim("code-506", "label-735");
        let view = to_rule_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
