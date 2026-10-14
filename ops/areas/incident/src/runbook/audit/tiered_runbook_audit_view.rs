use super::*;

use acme_kit::text;

/// How screens and exports show a tiered runbook audit.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditView {
    pub code: String,
    pub name: String,
    pub score_text: String,
    pub category: String,
    pub channel: String,
    pub approved: String,
    pub external_ref: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_tiered_runbook_audit_view(item: &TieredRunbookAudit) -> TieredRunbookAuditView {
    TieredRunbookAuditView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        score_text: format!("{:.2}", item.score),
        category: text::truncate(&item.category, 64),
        channel: item.channel.clone(),
        approved: format!("{}", item.approved),
        external_ref: item.external_ref.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_runbook_audit_views(items: &[TieredRunbookAudit]) -> Vec<TieredRunbookAuditView> {
    let mut views: Vec<TieredRunbookAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_runbook_audit_view(item));
    }
    views
}

pub fn sum_tiered_runbook_audit_view(view: &TieredRunbookAuditView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_score + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_runbook_audit("code-682", "name-850");
        let view = to_tiered_runbook_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_runbook_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
