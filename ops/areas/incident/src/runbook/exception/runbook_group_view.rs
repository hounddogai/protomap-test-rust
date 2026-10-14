use super::*;

use acme_kit::text;

/// How screens and exports show a runbook group.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookGroupView {
    pub code: String,
    pub title: String,
    pub scope_text: String,
    pub created_at: String,
    pub expires_at: String,
    pub floor_value_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_runbook_group_view(item: &RunbookGroup) -> RunbookGroupView {
    RunbookGroupView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        scope_text: item.scope.clone(),
        created_at: item.created_at.to_string(),
        expires_at: item.expires_at.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_group_views(items: &[RunbookGroup]) -> Vec<RunbookGroupView> {
    let mut views: Vec<RunbookGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_group_view(item));
    }
    views
}

pub fn sum_runbook_group_view(view: &RunbookGroupView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_expires_at as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_group("code-832", "title-134");
        let view = to_runbook_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
