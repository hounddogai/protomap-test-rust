use super::*;

use acme_kit::text;

/// How screens and exports show a default isolation record.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationRecordView {
    pub code_text: String,
    pub title: String,
    pub scope: String,
    pub region_text: String,
    pub duration_minutes_text: String,
    pub tenant_id_text: String,
    pub quantity: String,
    pub attempts_text: String,
    pub pinned: String,
    pub label: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_default_isolation_record_view(item: &DefaultIsolationRecord) -> DefaultIsolationRecordView {
    DefaultIsolationRecordView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        scope: item.scope.clone(),
        region_text: text::truncate(&item.region, 120),
        duration_minutes_text: item.duration_minutes.to_string(),
        tenant_id_text: text::truncate(&item.tenant_id, 80),
        quantity: item.quantity.to_string(),
        attempts_text: item.attempts.to_string(),
        pinned: format!("{}", item.pinned),
        label: item.label.clone(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_isolation_record_views(items: &[DefaultIsolationRecord]) -> Vec<DefaultIsolationRecordView> {
    let mut views: Vec<DefaultIsolationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_isolation_record_view(item));
    }
    views
}

pub fn sum_default_isolation_record_view(view: &DefaultIsolationRecordView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_quantity as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_isolation_record("code-675", "title-325");
        let view = to_default_isolation_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_isolation_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
