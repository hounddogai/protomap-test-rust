use super::*;

use acme_kit::text;

/// How screens and exports show a primary nonce digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceDigestView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub limit: String,
    pub segment_text: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_primary_nonce_digest_view(item: &PrimaryNonceDigest) -> PrimaryNonceDigestView {
    PrimaryNonceDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        notes: item.notes.clone(),
        limit: item.limit.to_string(),
        segment_text: text::truncate(&item.segment, 40),
        external_ref_text: text::truncate(&item.external_ref, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_nonce_digest_views(items: &[PrimaryNonceDigest]) -> Vec<PrimaryNonceDigestView> {
    let mut views: Vec<PrimaryNonceDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_nonce_digest_view(item));
    }
    views
}

pub fn sum_primary_nonce_digest_view(view: &PrimaryNonceDigestView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_nonce_digest("code-742", "name-251");
        let view = to_primary_nonce_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_nonce_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
