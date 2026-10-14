use super::*;

use acme_kit::text;

/// How screens and exports show a primary binding rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingRollupView {
    pub code: String,
    pub title: String,
    pub burst_text: String,
    pub source_text: String,
    pub label: String,
    pub budget_text: String,
    pub host_text: String,
    pub audience: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_primary_binding_rollup_view(item: &PrimaryBindingRollup) -> PrimaryBindingRollupView {
    PrimaryBindingRollupView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        burst_text: item.burst.to_string(),
        source_text: item.source.clone(),
        label: text::truncate(&item.label, 80),
        budget_text: format!("{:.2}", item.budget),
        host_text: item.host.clone(),
        audience: item.audience.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_binding_rollup_views(items: &[PrimaryBindingRollup]) -> Vec<PrimaryBindingRollupView> {
    let mut views: Vec<PrimaryBindingRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_binding_rollup_view(item));
    }
    views
}

pub fn sum_primary_binding_rollup_view(view: &PrimaryBindingRollupView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_burst as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_binding_rollup("code-162", "title-874");
        let view = to_primary_binding_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_binding_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
