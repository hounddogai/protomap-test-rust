use super::*;

use acme_kit::text;

/// How screens and exports show a runbook preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookPreferenceView {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub quantity: String,
    pub budget_text: String,
    pub ratio_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_runbook_preference_view(item: &RunbookPreference) -> RunbookPreferenceView {
    RunbookPreferenceView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        notes: item.notes.clone(),
        quantity: item.quantity.to_string(),
        budget_text: format!("{:.2}", item.budget),
        ratio_text: format!("{:.2}", item.ratio),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_preference_views(items: &[RunbookPreference]) -> Vec<RunbookPreferenceView> {
    let mut views: Vec<RunbookPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_preference_view(item));
    }
    views
}

pub fn sum_runbook_preference_view(view: &RunbookPreferenceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_budget + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_preference("code-204", "label-97");
        let view = to_runbook_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
