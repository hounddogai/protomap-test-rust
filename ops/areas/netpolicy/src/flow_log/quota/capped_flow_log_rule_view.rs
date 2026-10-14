use super::*;

use acme_kit::text;

/// How screens and exports show a capped flow log rule.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogRuleView {
    pub code: String,
    pub title: String,
    pub namespace_text: String,
    pub key_id: String,
    pub cost: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_capped_flow_log_rule_view(item: &CappedFlowLogRule) -> CappedFlowLogRuleView {
    CappedFlowLogRuleView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        namespace_text: item.namespace.clone(),
        key_id: item.key_id.clone(),
        cost: format!("{:.2}", item.cost),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_flow_log_rule_views(items: &[CappedFlowLogRule]) -> Vec<CappedFlowLogRuleView> {
    let mut views: Vec<CappedFlowLogRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_flow_log_rule_view(item));
    }
    views
}

pub fn sum_capped_flow_log_rule_view(view: &CappedFlowLogRuleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_flow_log_rule("code-185", "title-312");
        let view = to_capped_flow_log_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_flow_log_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
