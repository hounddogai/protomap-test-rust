use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageDigestView {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub retries: String,
    pub budget: String,
    pub revision: String,
    pub segment_text: String,
    pub limit: String,
    pub active: String,
    pub burst: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_regional_usage_digest_view(item: &RegionalUsageDigest) -> RegionalUsageDigestView {
    RegionalUsageDigestView {
        code: item.code.clone(),
        label: item.label.clone(),
        notes: item.notes.clone(),
        retries: item.retries.to_string(),
        budget: format!("{:.2}", item.budget),
        revision: item.revision.to_string(),
        segment_text: text::truncate(&item.segment, 80),
        limit: item.limit.to_string(),
        active: format!("{}", item.active),
        burst: item.burst.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_digest_views(items: &[RegionalUsageDigest]) -> Vec<RegionalUsageDigestView> {
    let mut views: Vec<RegionalUsageDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_digest_view(item));
    }
    views
}

pub fn sum_regional_usage_digest_view(view: &RegionalUsageDigestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_retries as f64 + parsed_budget + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_usage_digest("code-727", "label-484");
        let view = to_regional_usage_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_usage_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
