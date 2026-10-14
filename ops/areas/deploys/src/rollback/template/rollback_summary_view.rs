use super::*;

use acme_kit::text;

/// How screens and exports show a rollback summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSummaryView {
    pub code_text: String,
    pub label: String,
    pub quantity: String,
    pub namespace_text: String,
    pub threshold_text: String,
    pub cost_text: String,
    pub floor_value: String,
    pub minimum: String,
    pub cipher: String,
    pub scope_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_rollback_summary_view(item: &RollbackSummary) -> RollbackSummaryView {
    RollbackSummaryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        quantity: item.quantity.to_string(),
        namespace_text: text::truncate(&item.namespace, 120),
        threshold_text: format!("{:.2}", item.threshold),
        cost_text: format!("{:.2}", item.cost),
        floor_value: format!("{:.2}", item.floor_value),
        minimum: format!("{:.2}", item.minimum),
        cipher: item.cipher.clone(),
        scope_text: text::truncate(&item.scope, 40),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_summary_views(items: &[RollbackSummary]) -> Vec<RollbackSummaryView> {
    let mut views: Vec<RollbackSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_summary_view(item));
    }
    views
}

pub fn sum_rollback_summary_view(view: &RollbackSummaryView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_threshold + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_summary("code-666", "label-509");
        let view = to_rollback_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
