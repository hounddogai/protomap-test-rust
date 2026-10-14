use super::*;

use acme_kit::text;

/// How screens and exports show a primary health check rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHealthCheckRuleView {
    pub code_text: String,
    pub name_text: String,
    pub burst: String,
    pub phone_text: String,
    pub reviewer: String,
    pub cipher: String,
    pub tenant_id: String,
    pub key_id_text: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_primary_health_check_rule_view(item: &PrimaryHealthCheckRule) -> PrimaryHealthCheckRuleView {
    PrimaryHealthCheckRuleView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        burst: item.burst.to_string(),
        phone_text: text::truncate(&item.phone, 64),
        reviewer: text::truncate(&item.reviewer, 40),
        cipher: item.cipher.clone(),
        tenant_id: item.tenant_id.clone(),
        key_id_text: item.key_id.clone(),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_health_check_rule_views(items: &[PrimaryHealthCheckRule]) -> Vec<PrimaryHealthCheckRuleView> {
    let mut views: Vec<PrimaryHealthCheckRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_health_check_rule_view(item));
    }
    views
}

pub fn sum_primary_health_check_rule_view(view: &PrimaryHealthCheckRuleView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_health_check_rule("code-710", "name-185");
        let view = to_primary_health_check_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_health_check_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
