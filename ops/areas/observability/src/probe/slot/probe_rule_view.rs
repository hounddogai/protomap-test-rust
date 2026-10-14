use super::*;

use acme_kit::text;

/// How screens and exports show a probe rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRuleView {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub issuer_text: String,
    pub region: String,
    pub archived_text: String,
    pub currency: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_probe_rule_view(item: &ProbeRule) -> ProbeRuleView {
    ProbeRuleView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        scope: item.scope.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        region: item.region.clone(),
        archived_text: format!("{}", item.archived),
        currency: text::truncate(&item.currency, 80),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_rule_views(items: &[ProbeRule]) -> Vec<ProbeRuleView> {
    let mut views: Vec<ProbeRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_rule_view(item));
    }
    views
}

pub fn sum_probe_rule_view(view: &ProbeRuleView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_rule("code-274", "name-531");
        let view = to_probe_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
