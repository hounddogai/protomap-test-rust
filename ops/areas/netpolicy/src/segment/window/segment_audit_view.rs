use super::*;

/// How screens and exports show a segment audit.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAuditView {
    pub code_text: String,
    pub name: String,
    pub timezone_text: String,
    pub reviewer: String,
    pub margin_text: String,
    pub starts_at_text: String,
    pub duration_minutes: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_segment_audit_view(item: &SegmentAudit) -> SegmentAuditView {
    SegmentAuditView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        timezone_text: item.timezone.clone(),
        reviewer: item.reviewer.clone(),
        margin_text: format!("{:.2}", item.margin),
        starts_at_text: item.starts_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_audit_views(items: &[SegmentAudit]) -> Vec<SegmentAuditView> {
    let mut views: Vec<SegmentAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_audit_view(item));
    }
    views
}

pub fn sum_segment_audit_view(view: &SegmentAuditView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_margin + (parsed_starts_at as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_audit("code-130", "name-596");
        let view = to_segment_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
