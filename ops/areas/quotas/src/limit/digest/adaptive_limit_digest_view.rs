use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive limit digest.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitDigestView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub external_ref_text: String,
    pub description: String,
    pub cluster_text: String,
    pub name_text: String,
    pub batch_size: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_adaptive_limit_digest_view(item: &AdaptiveLimitDigest) -> AdaptiveLimitDigestView {
    AdaptiveLimitDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        external_ref_text: text::truncate(&item.external_ref, 40),
        description: item.description.clone(),
        cluster_text: item.cluster.clone(),
        name_text: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_limit_digest_views(items: &[AdaptiveLimitDigest]) -> Vec<AdaptiveLimitDigestView> {
    let mut views: Vec<AdaptiveLimitDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_limit_digest_view(item));
    }
    views
}

pub fn sum_adaptive_limit_digest_view(view: &AdaptiveLimitDigestView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ratio + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_limit_digest("code-85", "title-321");
        let view = to_adaptive_limit_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_limit_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
