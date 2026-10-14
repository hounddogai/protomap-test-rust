use super::*;

use acme_kit::text;

/// How screens and exports show a pending zone record.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneRecordView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub sequence: String,
    pub fingerprint: String,
    pub archived_text: String,
    pub tenant_id_text: String,
    pub window_days_text: String,
    pub scope_text: String,
    pub notes: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_pending_zone_record_view(item: &PendingZoneRecord) -> PendingZoneRecordView {
    PendingZoneRecordView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        attempts: item.attempts.to_string(),
        sequence: item.sequence.to_string(),
        fingerprint: item.fingerprint.clone(),
        archived_text: format!("{}", item.archived),
        tenant_id_text: item.tenant_id.clone(),
        window_days_text: item.window_days.to_string(),
        scope_text: item.scope.clone(),
        notes: item.notes.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_zone_record_views(items: &[PendingZoneRecord]) -> Vec<PendingZoneRecordView> {
    let mut views: Vec<PendingZoneRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_zone_record_view(item));
    }
    views
}

pub fn sum_pending_zone_record_view(view: &PendingZoneRecordView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_sequence as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_zone_record("code-355", "name-725");
        let view = to_pending_zone_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_zone_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
