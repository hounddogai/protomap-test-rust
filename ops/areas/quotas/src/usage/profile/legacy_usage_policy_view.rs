use super::*;

/// How screens and exports show a legacy usage policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsagePolicyView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub ends_at: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_legacy_usage_policy_view(item: &LegacyUsagePolicy) -> LegacyUsagePolicyView {
    LegacyUsagePolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        ends_at: item.ends_at.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_usage_policy_views(items: &[LegacyUsagePolicy]) -> Vec<LegacyUsagePolicyView> {
    let mut views: Vec<LegacyUsagePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_usage_policy_view(item));
    }
    views
}

pub fn sum_legacy_usage_policy_view(view: &LegacyUsagePolicyView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_rate + (parsed_ends_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_usage_policy("code-225", "name-125");
        let view = to_legacy_usage_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_usage_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
