use super::*;

use acme_kit::text;

/// How screens and exports show a rollback digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestView {
    pub code: String,
    pub label: String,
    pub owner_id_text: String,
    pub updated_at: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_rollback_digest_view(item: &RollbackDigest) -> RollbackDigestView {
    RollbackDigestView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 80),
        owner_id_text: item.owner_id.clone(),
        updated_at: item.updated_at.to_string(),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_digest_views(items: &[RollbackDigest]) -> Vec<RollbackDigestView> {
    let mut views: Vec<RollbackDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_digest_view(item));
    }
    views
}

pub fn sum_rollback_digest_view(view: &RollbackDigestView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_digest("code-898", "label-411");
        let view = to_rollback_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
