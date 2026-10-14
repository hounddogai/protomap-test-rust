use super::*;

use acme_kit::text;

/// How screens and exports show a default release digest.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseDigestView {
    pub code: String,
    pub name: String,
    pub limit_text: String,
    pub starts_at: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_default_release_digest_view(item: &DefaultReleaseDigest) -> DefaultReleaseDigestView {
    DefaultReleaseDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        limit_text: item.limit.to_string(),
        starts_at: item.starts_at.to_string(),
        region: text::truncate(&item.region, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_release_digest_views(items: &[DefaultReleaseDigest]) -> Vec<DefaultReleaseDigestView> {
    let mut views: Vec<DefaultReleaseDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_release_digest_view(item));
    }
    views
}

pub fn sum_default_release_digest_view(view: &DefaultReleaseDigestView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
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
        let item = new_default_release_digest("code-69", "name-117");
        let view = to_default_release_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_release_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
