use super::*;

use acme_kit::text;

/// How screens and exports show a pending nonce baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceBaselineView {
    pub code: String,
    pub name: String,
    pub limit: String,
    pub source: String,
    pub cluster: String,
    pub ratio: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_pending_nonce_baseline_view(item: &PendingNonceBaseline) -> PendingNonceBaselineView {
    PendingNonceBaselineView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        limit: item.limit.to_string(),
        source: item.source.clone(),
        cluster: text::truncate(&item.cluster, 64),
        ratio: format!("{:.2}", item.ratio),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_baseline_views(items: &[PendingNonceBaseline]) -> Vec<PendingNonceBaselineView> {
    let mut views: Vec<PendingNonceBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_baseline_view(item));
    }
    views
}

pub fn sum_pending_nonce_baseline_view(view: &PendingNonceBaselineView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ratio + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_nonce_baseline("code-313", "name-944");
        let view = to_pending_nonce_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_nonce_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
