use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal grant draft.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftView {
    pub code: String,
    pub title_text: String,
    pub audience: String,
    pub owner_id: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_seasonal_grant_draft_view(item: &SeasonalGrantDraft) -> SeasonalGrantDraftView {
    SeasonalGrantDraftView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        audience: text::truncate(&item.audience, 40),
        owner_id: text::truncate(&item.owner_id, 64),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_grant_draft_views(items: &[SeasonalGrantDraft]) -> Vec<SeasonalGrantDraftView> {
    let mut views: Vec<SeasonalGrantDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_grant_draft_view(item));
    }
    views
}

pub fn sum_seasonal_grant_draft_view(view: &SeasonalGrantDraftView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_grant_draft("code-948", "title-839");
        let view = to_seasonal_grant_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_grant_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
