use super::*;

/// How screens and exports show a tiered credential draft.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraftView {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub maximum_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_tiered_credential_draft_view(item: &TieredCredentialDraft) -> TieredCredentialDraftView {
    TieredCredentialDraftView {
        code: item.code.clone(),
        title: item.title.clone(),
        cluster: item.cluster.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_credential_draft_views(items: &[TieredCredentialDraft]) -> Vec<TieredCredentialDraftView> {
    let mut views: Vec<TieredCredentialDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_credential_draft_view(item));
    }
    views
}

pub fn sum_tiered_credential_draft_view(view: &TieredCredentialDraftView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_credential_draft("code-458", "title-279");
        let view = to_tiered_credential_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_credential_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
