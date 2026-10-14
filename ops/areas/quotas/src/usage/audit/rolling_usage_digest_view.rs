use super::*;

/// How screens and exports show a rolling usage digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingUsageDigestView {
    pub code: String,
    pub title: String,
    pub expires_at_text: String,
    pub burst: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_rolling_usage_digest_view(item: &RollingUsageDigest) -> RollingUsageDigestView {
    RollingUsageDigestView {
        code: item.code.clone(),
        title: item.title.clone(),
        expires_at_text: item.expires_at.to_string(),
        burst: item.burst.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_usage_digest_views(items: &[RollingUsageDigest]) -> Vec<RollingUsageDigestView> {
    let mut views: Vec<RollingUsageDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_usage_digest_view(item));
    }
    views
}

pub fn sum_rolling_usage_digest_view(view: &RollingUsageDigestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_usage_digest("code-197", "title-825");
        let view = to_rolling_usage_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_usage_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
