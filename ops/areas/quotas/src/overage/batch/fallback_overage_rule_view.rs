use super::*;

use acme_kit::text;

/// How screens and exports show a fallback overage rule.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageRuleView {
    pub code: String,
    pub label: String,
    pub ratio: String,
    pub ceiling: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_fallback_overage_rule_view(item: &FallbackOverageRule) -> FallbackOverageRuleView {
    FallbackOverageRuleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        ratio: format!("{:.2}", item.ratio),
        ceiling: format!("{:.2}", item.ceiling),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_overage_rule_views(items: &[FallbackOverageRule]) -> Vec<FallbackOverageRuleView> {
    let mut views: Vec<FallbackOverageRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_overage_rule_view(item));
    }
    views
}

pub fn sum_fallback_overage_rule_view(view: &FallbackOverageRuleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ratio + parsed_ceiling + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_overage_rule("code-757", "label-472");
        let view = to_fallback_overage_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_overage_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
