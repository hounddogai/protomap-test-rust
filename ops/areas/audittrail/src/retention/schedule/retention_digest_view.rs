use super::*;

/// How screens and exports show a retention digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDigestView {
    pub code_text: String,
    pub label: String,
    pub owner_id: String,
    pub reviewer: String,
    pub expires_at_text: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_retention_digest_view(item: &RetentionDigest) -> RetentionDigestView {
    RetentionDigestView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        owner_id: item.owner_id.clone(),
        reviewer: item.reviewer.clone(),
        expires_at_text: item.expires_at.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_digest_views(items: &[RetentionDigest]) -> Vec<RetentionDigestView> {
    let mut views: Vec<RetentionDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_digest_view(item));
    }
    views
}

pub fn sum_retention_digest_view(view: &RetentionDigestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_digest("code-497", "label-80");
        let view = to_retention_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
