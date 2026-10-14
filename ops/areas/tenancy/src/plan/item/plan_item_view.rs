use super::*;

use acme_kit::text;

/// How screens and exports show a plan item.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanItemView {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub ratio: String,
    pub notes_text: String,
    pub revision: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_plan_item_view(item: &PlanItem) -> PlanItemView {
    PlanItemView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        reviewer: item.reviewer.clone(),
        ratio: format!("{:.2}", item.ratio),
        notes_text: item.notes.clone(),
        revision: item.revision.to_string(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_item_views(items: &[PlanItem]) -> Vec<PlanItemView> {
    let mut views: Vec<PlanItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_item_view(item));
    }
    views
}

pub fn sum_plan_item_view(view: &PlanItemView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ratio + (parsed_revision as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_item("code-501", "name-223");
        let view = to_plan_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
