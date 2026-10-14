use super::*;

/// How screens and exports show a usage rule.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageRuleView {
    pub code_text: String,
    pub title: String,
    pub segment: String,
    pub quota_units_text: String,
    pub currency: String,
    pub ceiling_text: String,
    pub region: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_usage_rule_view(item: &UsageRule) -> UsageRuleView {
    UsageRuleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        segment: item.segment.clone(),
        quota_units_text: item.quota_units.to_string(),
        currency: item.currency.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        region: item.region.clone(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_rule_views(items: &[UsageRule]) -> Vec<UsageRuleView> {
    let mut views: Vec<UsageRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_rule_view(item));
    }
    views
}

pub fn sum_usage_rule_view(view: &UsageRuleView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_rule("code-941", "title-10");
        let view = to_usage_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
