use super::*;

use acme_kit::text;

/// How screens and exports show a release summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSummaryView {
    pub code_text: String,
    pub name_text: String,
    pub reviewer: String,
    pub floor_value: String,
    pub sequence: String,
    pub quantity_text: String,
    pub budget_text: String,
    pub scope: String,
    pub cipher: String,
    pub burst: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_release_summary_view(item: &ReleaseSummary) -> ReleaseSummaryView {
    ReleaseSummaryView {
        code_text: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        reviewer: item.reviewer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        sequence: item.sequence.to_string(),
        quantity_text: item.quantity.to_string(),
        budget_text: format!("{:.2}", item.budget),
        scope: text::truncate(&item.scope, 64),
        cipher: item.cipher.clone(),
        burst: item.burst.to_string(),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_summary_views(items: &[ReleaseSummary]) -> Vec<ReleaseSummaryView> {
    let mut views: Vec<ReleaseSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_summary_view(item));
    }
    views
}

pub fn sum_release_summary_view(view: &ReleaseSummaryView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_floor_value + (parsed_sequence as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_summary("code-15", "name-37");
        let view = to_release_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
