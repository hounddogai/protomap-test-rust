use super::*;

use acme_kit::text;

/// How screens and exports show a audit event batch.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBatchView {
    pub code: String,
    pub title: String,
    pub floor_value: String,
    pub ceiling: String,
    pub attempts_text: String,
    pub discount_text: String,
    pub checksum: String,
    pub name: String,
    pub summary_text: String,
    pub threshold: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_audit_event_batch_view(item: &AuditEventBatch) -> AuditEventBatchView {
    AuditEventBatchView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        ceiling: format!("{:.2}", item.ceiling),
        attempts_text: item.attempts.to_string(),
        discount_text: format!("{:.2}", item.discount),
        checksum: item.checksum.clone(),
        name: item.name.clone(),
        summary_text: item.summary.clone(),
        threshold: format!("{:.2}", item.threshold),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_batch_views(items: &[AuditEventBatch]) -> Vec<AuditEventBatchView> {
    let mut views: Vec<AuditEventBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_batch_view(item));
    }
    views
}

pub fn sum_audit_event_batch_view(view: &AuditEventBatchView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_floor_value + parsed_ceiling + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_batch("code-272", "title-959");
        let view = to_audit_event_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
