use super::*;

use acme_kit::text;

/// How screens and exports show a federation draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationDraftView {
    pub code: String,
    pub name_text: String,
    pub rotation_days: String,
    pub limit: String,
    pub summary: String,
    pub namespace_text: String,
    pub status_label: String,
}

pub fn to_federation_draft_view(item: &FederationDraft) -> FederationDraftView {
    FederationDraftView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        rotation_days: item.rotation_days.to_string(),
        limit: item.limit.to_string(),
        summary: text::truncate(&item.summary, 64),
        namespace_text: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_draft_views(items: &[FederationDraft]) -> Vec<FederationDraftView> {
    let mut views: Vec<FederationDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_draft_view(item));
    }
    views
}

pub fn sum_federation_draft_view(view: &FederationDraftView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_draft("code-585", "name-811");
        let view = to_federation_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
