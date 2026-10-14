use super::*;

use acme_kit::text;

/// How screens and exports show a pending nonce record.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceRecordView {
    pub code_text: String,
    pub name_text: String,
    pub floor_value: String,
    pub limit: String,
    pub margin: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_pending_nonce_record_view(item: &PendingNonceRecord) -> PendingNonceRecordView {
    PendingNonceRecordView {
        code_text: text::truncate(&item.code, 64),
        name_text: text::truncate(&item.name, 64),
        floor_value: format!("{:.2}", item.floor_value),
        limit: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        owner_id: text::truncate(&item.owner_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_nonce_record_views(items: &[PendingNonceRecord]) -> Vec<PendingNonceRecordView> {
    let mut views: Vec<PendingNonceRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_nonce_record_view(item));
    }
    views
}

pub fn sum_pending_nonce_record_view(view: &PendingNonceRecordView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_floor_value + (parsed_limit as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_nonce_record("code-891", "name-600");
        let view = to_pending_nonce_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_nonce_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
