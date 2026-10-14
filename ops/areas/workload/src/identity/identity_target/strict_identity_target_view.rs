use super::*;

use acme_kit::text;

/// How screens and exports show a strict identity target.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTargetView {
    pub code_text: String,
    pub title_text: String,
    pub revision: String,
    pub namespace_text: String,
    pub archived_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_strict_identity_target_view(item: &StrictIdentityTarget) -> StrictIdentityTargetView {
    StrictIdentityTargetView {
        code_text: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 40),
        revision: item.revision.to_string(),
        namespace_text: item.namespace.clone(),
        archived_text: format!("{}", item.archived),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_identity_target_views(items: &[StrictIdentityTarget]) -> Vec<StrictIdentityTargetView> {
    let mut views: Vec<StrictIdentityTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_identity_target_view(item));
    }
    views
}

pub fn sum_strict_identity_target_view(view: &StrictIdentityTargetView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_identity_target("code-134", "title-952");
        let view = to_strict_identity_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_identity_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
