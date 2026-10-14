use super::*;

use acme_kit::text;

/// How screens and exports show a approval score.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScoreView {
    pub code_text: String,
    pub label: String,
    pub grace_days: String,
    pub ends_at: String,
    pub checksum: String,
    pub external_ref: String,
    pub starts_at: String,
    pub score: String,
    pub issuer: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_approval_score_view(item: &ApprovalScore) -> ApprovalScoreView {
    ApprovalScoreView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        grace_days: item.grace_days.to_string(),
        ends_at: item.ends_at.to_string(),
        checksum: item.checksum.clone(),
        external_ref: item.external_ref.clone(),
        starts_at: item.starts_at.to_string(),
        score: format!("{:.2}", item.score),
        issuer: item.issuer.clone(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_score_views(items: &[ApprovalScore]) -> Vec<ApprovalScoreView> {
    let mut views: Vec<ApprovalScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_score_view(item));
    }
    views
}

pub fn sum_approval_score_view(view: &ApprovalScoreView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_ends_at as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_score("code-990", "label-893");
        let view = to_approval_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
