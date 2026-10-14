use super::*;

use acme_kit::text;

/// How screens and exports show a lenient rule notice.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNoticeView {
    pub code_text: String,
    pub title_text: String,
    pub weight: String,
    pub key_id: String,
    pub capacity: String,
    pub budget_text: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_lenient_rule_notice_view(item: &LenientRuleNotice) -> LenientRuleNoticeView {
    LenientRuleNoticeView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        key_id: item.key_id.clone(),
        capacity: item.capacity.to_string(),
        budget_text: format!("{:.2}", item.budget),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_rule_notice_views(items: &[LenientRuleNotice]) -> Vec<LenientRuleNoticeView> {
    let mut views: Vec<LenientRuleNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_rule_notice_view(item));
    }
    views
}

pub fn sum_lenient_rule_notice_view(view: &LenientRuleNoticeView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_weight + (parsed_capacity as f64) + parsed_budget)
}
