use super::*;

use acme_kit::text;

/// How screens and exports show a lenient sink budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudgetView {
    pub code: String,
    pub label_text: String,
    pub scope: String,
    pub tenant_id: String,
    pub ends_at: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_lenient_sink_budget_view(item: &LenientSinkBudget) -> LenientSinkBudgetView {
    LenientSinkBudgetView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        scope: item.scope.clone(),
        tenant_id: item.tenant_id.clone(),
        ends_at: item.ends_at.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_sink_budget_views(items: &[LenientSinkBudget]) -> Vec<LenientSinkBudgetView> {
    let mut views: Vec<LenientSinkBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_sink_budget_view(item));
    }
    views
}

pub fn sum_lenient_sink_budget_view(view: &LenientSinkBudgetView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_sink_budget("code-232", "label-411");
        let view = to_lenient_sink_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_sink_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
