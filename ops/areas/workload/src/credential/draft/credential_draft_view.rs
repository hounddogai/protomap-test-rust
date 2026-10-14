use super::*;

use acme_kit::text;

/// How screens and exports show a credential draft.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialDraftView {
    pub code: String,
    pub name_text: String,
    pub summary_text: String,
    pub discount: String,
    pub score: String,
    pub description: String,
    pub category: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_credential_draft_view(item: &CredentialDraft) -> CredentialDraftView {
    CredentialDraftView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        summary_text: text::truncate(&item.summary, 64),
        discount: format!("{:.2}", item.discount),
        score: format!("{:.2}", item.score),
        description: item.description.clone(),
        category: item.category.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_draft_views(items: &[CredentialDraft]) -> Vec<CredentialDraftView> {
    let mut views: Vec<CredentialDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_draft_view(item));
    }
    views
}

pub fn sum_credential_draft_view(view: &CredentialDraftView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_discount + parsed_score + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_draft("code-242", "name-451");
        let view = to_credential_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
