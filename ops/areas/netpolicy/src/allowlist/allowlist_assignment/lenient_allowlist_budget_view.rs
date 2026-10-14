use super::*;

use acme_kit::text;

/// How screens and exports show a lenient allowlist budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistBudgetView {
    pub code_text: String,
    pub name_text: String,
    pub margin: String,
    pub label: String,
    pub region: String,
    pub floor_value: String,
    pub country_text: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_lenient_allowlist_budget_view(item: &LenientAllowlistBudget) -> LenientAllowlistBudgetView {
    LenientAllowlistBudgetView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        label: text::truncate(&item.label, 40),
        region: text::truncate(&item.region, 40),
        floor_value: format!("{:.2}", item.floor_value),
        country_text: item.country.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_allowlist_budget_views(items: &[LenientAllowlistBudget]) -> Vec<LenientAllowlistBudgetView> {
    let mut views: Vec<LenientAllowlistBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_allowlist_budget_view(item));
    }
    views
}

pub fn sum_lenient_allowlist_budget_view(view: &LenientAllowlistBudgetView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_margin + parsed_floor_value + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_allowlist_budget("code-563", "name-118");
        let view = to_lenient_allowlist_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_allowlist_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
