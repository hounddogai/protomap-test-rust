use super::*;

use acme_kit::text;

/// How screens and exports show a strict rule request.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleRequestView {
    pub code: String,
    pub name: String,
    pub checksum_text: String,
    pub currency: String,
    pub archived: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_strict_rule_request_view(item: &StrictRuleRequest) -> StrictRuleRequestView {
    StrictRuleRequestView {
        code: item.code.clone(),
        name: item.name.clone(),
        checksum_text: item.checksum.clone(),
        currency: text::truncate(&item.currency, 64),
        archived: format!("{}", item.archived),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_rule_request_views(items: &[StrictRuleRequest]) -> Vec<StrictRuleRequestView> {
    let mut views: Vec<StrictRuleRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_rule_request_view(item));
    }
    views
}

pub fn sum_strict_rule_request_view(view: &StrictRuleRequestView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_rule_request("code-158", "name-650");
        let view = to_strict_rule_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_rule_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
