use super::*;

/// How screens and exports show a ttl policy tier.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTierView {
    pub code_text: String,
    pub name: String,
    pub ceiling: String,
    pub maximum_text: String,
    pub grace_days_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_ttl_policy_tier_view(item: &TtlPolicyTier) -> TtlPolicyTierView {
    TtlPolicyTierView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        maximum_text: format!("{:.2}", item.maximum),
        grace_days_text: item.grace_days.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_tier_views(items: &[TtlPolicyTier]) -> Vec<TtlPolicyTierView> {
    let mut views: Vec<TtlPolicyTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_tier_view(item));
    }
    views
}

pub fn sum_ttl_policy_tier_view(view: &TtlPolicyTierView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ceiling + parsed_maximum + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_tier("code-578", "name-870");
        let view = to_ttl_policy_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
