use super::*;

/// How screens and exports show a primary binding rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRuleView {
    pub code: String,
    pub label: String,
    pub host_text: String,
    pub principal_text: String,
    pub source: String,
    pub score: String,
    pub namespace: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_primary_binding_rule_view(item: &PrimaryBindingRule) -> PrimaryBindingRuleView {
    PrimaryBindingRuleView {
        code: item.code.clone(),
        label: item.label.clone(),
        host_text: item.host.clone(),
        principal_text: item.principal.clone(),
        source: item.source.clone(),
        score: format!("{:.2}", item.score),
        namespace: item.namespace.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_binding_rule_views(items: &[PrimaryBindingRule]) -> Vec<PrimaryBindingRuleView> {
    let mut views: Vec<PrimaryBindingRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_binding_rule_view(item));
    }
    views
}

pub fn sum_primary_binding_rule_view(view: &PrimaryBindingRuleView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_score + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_binding_rule("code-897", "label-782");
        let view = to_primary_binding_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_binding_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
