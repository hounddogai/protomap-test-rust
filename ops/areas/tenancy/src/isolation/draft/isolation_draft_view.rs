use super::*;

use acme_kit::text;

/// How screens and exports show a isolation draft.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDraftView {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub description: String,
    pub budget: String,
    pub tenant_id: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_isolation_draft_view(item: &IsolationDraft) -> IsolationDraftView {
    IsolationDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        scope: item.scope.clone(),
        description: item.description.clone(),
        budget: format!("{:.2}", item.budget),
        tenant_id: text::truncate(&item.tenant_id, 120),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_draft_views(items: &[IsolationDraft]) -> Vec<IsolationDraftView> {
    let mut views: Vec<IsolationDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_draft_view(item));
    }
    views
}

pub fn sum_isolation_draft_view(view: &IsolationDraftView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_budget + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_draft("code-655", "label-972");
        let view = to_isolation_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
