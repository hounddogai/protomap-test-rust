use super::*;

use acme_kit::text;

/// How screens and exports show a isolation audit.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationAuditView {
    pub code: String,
    pub name_text: String,
    pub visible_text: String,
    pub threshold: String,
    pub limit: String,
    pub window_days_text: String,
    pub score_text: String,
    pub unit: String,
    pub principal_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_isolation_audit_view(item: &IsolationAudit) -> IsolationAuditView {
    IsolationAuditView {
        code: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 120),
        visible_text: format!("{}", item.visible),
        threshold: format!("{:.2}", item.threshold),
        limit: item.limit.to_string(),
        window_days_text: item.window_days.to_string(),
        score_text: format!("{:.2}", item.score),
        unit: item.unit.clone(),
        principal_text: item.principal.clone(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_audit_views(items: &[IsolationAudit]) -> Vec<IsolationAuditView> {
    let mut views: Vec<IsolationAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_audit_view(item));
    }
    views
}

pub fn sum_isolation_audit_view(view: &IsolationAuditView) -> Result<f64, String> {
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
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_threshold + (parsed_limit as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_audit("code-924", "name-400");
        let view = to_isolation_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
