use super::*;

use acme_kit::text;

/// How screens and exports show a query draft.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraftView {
    pub code: String,
    pub title_text: String,
    pub ceiling_text: String,
    pub pinned_text: String,
    pub source_text: String,
    pub description: String,
    pub visible: String,
    pub principal: String,
    pub duration_minutes: String,
    pub budget: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_query_draft_view(item: &QueryDraft) -> QueryDraftView {
    QueryDraftView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        pinned_text: format!("{}", item.pinned),
        source_text: item.source.clone(),
        description: item.description.clone(),
        visible: format!("{}", item.visible),
        principal: text::truncate(&item.principal, 120),
        duration_minutes: item.duration_minutes.to_string(),
        budget: format!("{:.2}", item.budget),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_draft_views(items: &[QueryDraft]) -> Vec<QueryDraftView> {
    let mut views: Vec<QueryDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_draft_view(item));
    }
    views
}

pub fn sum_query_draft_view(view: &QueryDraftView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ceiling + (parsed_duration_minutes as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_draft("code-881", "title-267");
        let view = to_query_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
