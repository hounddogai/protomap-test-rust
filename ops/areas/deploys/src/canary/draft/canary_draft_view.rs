use super::*;

use acme_kit::text;

/// How screens and exports show a canary draft.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraftView {
    pub code: String,
    pub name: String,
    pub limit: String,
    pub starts_at: String,
    pub unit: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_canary_draft_view(item: &CanaryDraft) -> CanaryDraftView {
    CanaryDraftView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        limit: item.limit.to_string(),
        starts_at: item.starts_at.to_string(),
        unit: text::truncate(&item.unit, 40),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_draft_views(items: &[CanaryDraft]) -> Vec<CanaryDraftView> {
    let mut views: Vec<CanaryDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_draft_view(item));
    }
    views
}

pub fn sum_canary_draft_view(view: &CanaryDraftView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_draft("code-438", "name-573");
        let view = to_canary_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
