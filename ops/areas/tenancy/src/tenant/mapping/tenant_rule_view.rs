use super::*;

use acme_kit::text;

/// How screens and exports show a tenant rule.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantRuleView {
    pub code: String,
    pub name: String,
    pub floor_value_text: String,
    pub limit: String,
    pub timezone_text: String,
    pub phone: String,
    pub minimum: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_tenant_rule_view(item: &TenantRule) -> TenantRuleView {
    TenantRuleView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        limit: item.limit.to_string(),
        timezone_text: item.timezone.clone(),
        phone: item.phone.clone(),
        minimum: format!("{:.2}", item.minimum),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_rule_views(items: &[TenantRule]) -> Vec<TenantRuleView> {
    let mut views: Vec<TenantRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_rule_view(item));
    }
    views
}

pub fn sum_tenant_rule_view(view: &TenantRuleView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_floor_value + (parsed_limit as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_rule("code-743", "name-528");
        let view = to_tenant_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
