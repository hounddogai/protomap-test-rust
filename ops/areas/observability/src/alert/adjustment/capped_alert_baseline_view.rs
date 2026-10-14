use super::*;

use acme_kit::text;

/// How screens and exports show a capped alert baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAlertBaselineView {
    pub code_text: String,
    pub title: String,
    pub priority_text: String,
    pub starts_at_text: String,
    pub rate: String,
    pub port: String,
    pub limit: String,
    pub active: String,
    pub name_text: String,
    pub reviewer_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_capped_alert_baseline_view(item: &CappedAlertBaseline) -> CappedAlertBaselineView {
    CappedAlertBaselineView {
        code_text: text::truncate(&item.code, 80),
        title: item.title.clone(),
        priority_text: item.priority.to_string(),
        starts_at_text: item.starts_at.to_string(),
        rate: format!("{:.2}", item.rate),
        port: item.port.to_string(),
        limit: item.limit.to_string(),
        active: format!("{}", item.active),
        name_text: item.name.clone(),
        reviewer_text: item.reviewer.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_alert_baseline_views(items: &[CappedAlertBaseline]) -> Vec<CappedAlertBaselineView> {
    let mut views: Vec<CappedAlertBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_alert_baseline_view(item));
    }
    views
}

pub fn sum_capped_alert_baseline_view(view: &CappedAlertBaselineView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_starts_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_alert_baseline("code-411", "title-853");
        let view = to_capped_alert_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_alert_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
