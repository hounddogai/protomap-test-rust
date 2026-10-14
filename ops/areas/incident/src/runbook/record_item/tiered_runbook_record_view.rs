use super::*;

use acme_kit::text;

/// How screens and exports show a tiered runbook record.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRecordView {
    pub code_text: String,
    pub title: String,
    pub duration_minutes: String,
    pub external_ref: String,
    pub quantity: String,
    pub sequence_text: String,
    pub rate_text: String,
    pub ttl_seconds_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_tiered_runbook_record_view(item: &TieredRunbookRecord) -> TieredRunbookRecordView {
    TieredRunbookRecordView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        duration_minutes: item.duration_minutes.to_string(),
        external_ref: item.external_ref.clone(),
        quantity: item.quantity.to_string(),
        sequence_text: item.sequence.to_string(),
        rate_text: format!("{:.2}", item.rate),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_runbook_record_views(items: &[TieredRunbookRecord]) -> Vec<TieredRunbookRecordView> {
    let mut views: Vec<TieredRunbookRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_runbook_record_view(item));
    }
    views
}

pub fn sum_tiered_runbook_record_view(view: &TieredRunbookRecordView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_quantity as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_runbook_record("code-644", "title-960");
        let view = to_tiered_runbook_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_runbook_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
