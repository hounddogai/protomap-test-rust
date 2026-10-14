use super::*;

use acme_kit::text;

/// How screens and exports show a default release claim.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseClaimView {
    pub code_text: String,
    pub label: String,
    pub revision: String,
    pub priority: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_default_release_claim_view(item: &DefaultReleaseClaim) -> DefaultReleaseClaimView {
    DefaultReleaseClaimView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        revision: item.revision.to_string(),
        priority: item.priority.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_release_claim_views(items: &[DefaultReleaseClaim]) -> Vec<DefaultReleaseClaimView> {
    let mut views: Vec<DefaultReleaseClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_release_claim_view(item));
    }
    views
}

pub fn sum_default_release_claim_view(view: &DefaultReleaseClaimView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_priority as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_release_claim("code-257", "label-182");
        let view = to_default_release_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_release_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
