use super::*;

use acme_kit::text;

/// How screens and exports show a redaction baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaselineView {
    pub code: String,
    pub label_text: String,
    pub discount_text: String,
    pub margin: String,
    pub budget: String,
    pub approved_text: String,
    pub namespace_text: String,
    pub principal: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_redaction_baseline_view(item: &RedactionBaseline) -> RedactionBaselineView {
    RedactionBaselineView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        discount_text: format!("{:.2}", item.discount),
        margin: format!("{:.2}", item.margin),
        budget: format!("{:.2}", item.budget),
        approved_text: format!("{}", item.approved),
        namespace_text: item.namespace.clone(),
        principal: item.principal.clone(),
        region_text: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_baseline_views(items: &[RedactionBaseline]) -> Vec<RedactionBaselineView> {
    let mut views: Vec<RedactionBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_baseline_view(item));
    }
    views
}

pub fn sum_redaction_baseline_view(view: &RedactionBaselineView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_discount + parsed_margin + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_baseline("code-869", "label-668");
        let view = to_redaction_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
