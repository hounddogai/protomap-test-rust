use super::*;

use acme_kit::text;

/// How screens and exports show a fallback rule item.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleItemView {
    pub code: String,
    pub title_text: String,
    pub margin: String,
    pub notes: String,
    pub cost: String,
    pub namespace_text: String,
    pub floor_value: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_fallback_rule_item_view(item: &FallbackRuleItem) -> FallbackRuleItemView {
    FallbackRuleItemView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        notes: item.notes.clone(),
        cost: format!("{:.2}", item.cost),
        namespace_text: item.namespace.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_rule_item_views(items: &[FallbackRuleItem]) -> Vec<FallbackRuleItemView> {
    let mut views: Vec<FallbackRuleItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_rule_item_view(item));
    }
    views
}

pub fn sum_fallback_rule_item_view(view: &FallbackRuleItemView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_margin + parsed_cost + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_rule_item("code-599", "title-669");
        let view = to_fallback_rule_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_rule_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
