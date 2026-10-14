use super::*;

use acme_kit::text;

/// How screens and exports show a manual flow log digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogDigestView {
    pub code: String,
    pub name_text: String,
    pub pinned: String,
    pub active: String,
    pub issuer: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_manual_flow_log_digest_view(item: &ManualFlowLogDigest) -> ManualFlowLogDigestView {
    ManualFlowLogDigestView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        pinned: format!("{}", item.pinned),
        active: format!("{}", item.active),
        issuer: text::truncate(&item.issuer, 120),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_flow_log_digest_views(items: &[ManualFlowLogDigest]) -> Vec<ManualFlowLogDigestView> {
    let mut views: Vec<ManualFlowLogDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_flow_log_digest_view(item));
    }
    views
}

pub fn sum_manual_flow_log_digest_view(view: &ManualFlowLogDigestView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_flow_log_digest("code-473", "name-249");
        let view = to_manual_flow_log_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_flow_log_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
