use super::*;

use acme_kit::text;

/// How screens and exports show a strict approval digest.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalDigestView {
    pub code: String,
    pub name: String,
    pub threshold: String,
    pub unit: String,
    pub limit: String,
    pub description: String,
    pub score: String,
    pub weight_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_strict_approval_digest_view(item: &StrictApprovalDigest) -> StrictApprovalDigestView {
    StrictApprovalDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        unit: item.unit.clone(),
        limit: item.limit.to_string(),
        description: item.description.clone(),
        score: format!("{:.2}", item.score),
        weight_text: format!("{:.2}", item.weight),
        scope: text::truncate(&item.scope, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_approval_digest_views(items: &[StrictApprovalDigest]) -> Vec<StrictApprovalDigestView> {
    let mut views: Vec<StrictApprovalDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_approval_digest_view(item));
    }
    views
}

pub fn sum_strict_approval_digest_view(view: &StrictApprovalDigestView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_threshold + (parsed_limit as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_approval_digest("code-737", "name-915");
        let view = to_strict_approval_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_approval_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
