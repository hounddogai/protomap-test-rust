use super::*;

/// How screens and exports show a pending retention digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestView {
    pub code: String,
    pub title_text: String,
    pub attempts: String,
    pub maximum_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_pending_retention_digest_view(item: &PendingRetentionDigest) -> PendingRetentionDigestView {
    PendingRetentionDigestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        attempts: item.attempts.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_retention_digest_views(items: &[PendingRetentionDigest]) -> Vec<PendingRetentionDigestView> {
    let mut views: Vec<PendingRetentionDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_retention_digest_view(item));
    }
    views
}

pub fn sum_pending_retention_digest_view(view: &PendingRetentionDigestView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_retention_digest("code-427", "title-762");
        let view = to_pending_retention_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_retention_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
