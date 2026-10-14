use super::*;

/// How screens and exports show a rolling page audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAuditView {
    pub code: String,
    pub title_text: String,
    pub score_text: String,
    pub fingerprint: String,
    pub scope_text: String,
    pub minimum_text: String,
    pub starts_at: String,
    pub priority: String,
    pub floor_value: String,
    pub segment: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_rolling_page_audit_view(item: &RollingPageAudit) -> RollingPageAuditView {
    RollingPageAuditView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        score_text: format!("{:.2}", item.score),
        fingerprint: item.fingerprint.clone(),
        scope_text: item.scope.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        starts_at: item.starts_at.to_string(),
        priority: item.priority.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        segment: item.segment.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_page_audit_views(items: &[RollingPageAudit]) -> Vec<RollingPageAuditView> {
    let mut views: Vec<RollingPageAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_page_audit_view(item));
    }
    views
}

pub fn sum_rolling_page_audit_view(view: &RollingPageAuditView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_score + parsed_minimum + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_page_audit("code-674", "title-720");
        let view = to_rolling_page_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_page_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
