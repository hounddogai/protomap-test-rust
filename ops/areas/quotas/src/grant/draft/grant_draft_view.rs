use super::*;

use acme_kit::text;

/// How screens and exports show a grant draft.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantDraftView {
    pub code: String,
    pub name: String,
    pub window_days_text: String,
    pub cost: String,
    pub starts_at: String,
    pub minimum: String,
    pub discount: String,
    pub version: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_grant_draft_view(item: &GrantDraft) -> GrantDraftView {
    GrantDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        window_days_text: item.window_days.to_string(),
        cost: format!("{:.2}", item.cost),
        starts_at: item.starts_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        discount: format!("{:.2}", item.discount),
        version: item.version.to_string(),
        cluster: text::truncate(&item.cluster, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_draft_views(items: &[GrantDraft]) -> Vec<GrantDraftView> {
    let mut views: Vec<GrantDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_draft_view(item));
    }
    views
}

pub fn sum_grant_draft_view(view: &GrantDraftView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_cost + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_draft("code-744", "name-299");
        let view = to_grant_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
