use super::*;

use acme_kit::text;

/// How screens and exports show a strict rule draft.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraftView {
    pub code_text: String,
    pub title: String,
    pub currency: String,
    pub cost_text: String,
    pub floor_value_text: String,
    pub channel_text: String,
    pub weight: String,
    pub updated_at: String,
    pub scope: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_strict_rule_draft_view(item: &StrictRuleDraft) -> StrictRuleDraftView {
    StrictRuleDraftView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        currency: text::truncate(&item.currency, 64),
        cost_text: format!("{:.2}", item.cost),
        floor_value_text: format!("{:.2}", item.floor_value),
        channel_text: text::truncate(&item.channel, 120),
        weight: format!("{:.2}", item.weight),
        updated_at: item.updated_at.to_string(),
        scope: item.scope.clone(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_rule_draft_views(items: &[StrictRuleDraft]) -> Vec<StrictRuleDraftView> {
    let mut views: Vec<StrictRuleDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_rule_draft_view(item));
    }
    views
}

pub fn sum_strict_rule_draft_view(view: &StrictRuleDraftView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_cost + parsed_floor_value + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_rule_draft("code-277", "title-573");
        let view = to_strict_rule_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_rule_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
