use super::*;

use acme_kit::text;

/// How screens and exports show a grant baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantBaselineView {
    pub code: String,
    pub label_text: String,
    pub ttl_seconds: String,
    pub contact_email_text: String,
    pub segment: String,
    pub limit_text: String,
    pub expires_at: String,
    pub score_text: String,
    pub port: String,
    pub rotation_days: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_grant_baseline_view(item: &GrantBaseline) -> GrantBaselineView {
    GrantBaselineView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        contact_email_text: item.contact_email.clone(),
        segment: text::truncate(&item.segment, 120),
        limit_text: item.limit.to_string(),
        expires_at: item.expires_at.to_string(),
        score_text: format!("{:.2}", item.score),
        port: item.port.to_string(),
        rotation_days: item.rotation_days.to_string(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_baseline_views(items: &[GrantBaseline]) -> Vec<GrantBaselineView> {
    let mut views: Vec<GrantBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_baseline_view(item));
    }
    views
}

pub fn sum_grant_baseline_view(view: &GrantBaselineView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_limit as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_baseline("code-468", "label-755");
        let view = to_grant_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
