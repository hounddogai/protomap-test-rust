use super::*;

/// How screens and exports show a dns record rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleView {
    pub code_text: String,
    pub name: String,
    pub ttl_seconds: String,
    pub limit: String,
    pub rotation_days: String,
    pub floor_value: String,
    pub created_at: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_dns_record_rule_view(item: &DnsRecordRule) -> DnsRecordRuleView {
    DnsRecordRuleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        limit: item.limit.to_string(),
        rotation_days: item.rotation_days.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        created_at: item.created_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_rule_views(items: &[DnsRecordRule]) -> Vec<DnsRecordRuleView> {
    let mut views: Vec<DnsRecordRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_rule_view(item));
    }
    views
}

pub fn sum_dns_record_rule_view(view: &DnsRecordRuleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_limit as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_rule("code-110", "name-863");
        let view = to_dns_record_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
