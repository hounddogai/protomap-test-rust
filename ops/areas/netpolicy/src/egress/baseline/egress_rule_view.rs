use super::*;

use acme_kit::text;

/// How screens and exports show a egress rule.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRuleView {
    pub code: String,
    pub label: String,
    pub version_text: String,
    pub visible: String,
    pub active: String,
    pub summary_text: String,
    pub channel: String,
    pub discount_text: String,
    pub rate: String,
    pub category: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_egress_rule_view(item: &EgressRule) -> EgressRuleView {
    EgressRuleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        version_text: item.version.to_string(),
        visible: format!("{}", item.visible),
        active: format!("{}", item.active),
        summary_text: item.summary.clone(),
        channel: item.channel.clone(),
        discount_text: format!("{:.2}", item.discount),
        rate: format!("{:.2}", item.rate),
        category: text::truncate(&item.category, 40),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_rule_views(items: &[EgressRule]) -> Vec<EgressRuleView> {
    let mut views: Vec<EgressRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_rule_view(item));
    }
    views
}

pub fn sum_egress_rule_view(view: &EgressRuleView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_version as f64 + parsed_discount + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_rule("code-842", "label-743");
        let view = to_egress_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
