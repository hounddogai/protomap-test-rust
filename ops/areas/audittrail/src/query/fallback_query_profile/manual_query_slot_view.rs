use super::*;

use acme_kit::text;

/// How screens and exports show a manual query slot.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlotView {
    pub code: String,
    pub title: String,
    pub attempts: String,
    pub expires_at: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_manual_query_slot_view(item: &ManualQuerySlot) -> ManualQuerySlotView {
    ManualQuerySlotView {
        code: item.code.clone(),
        title: item.title.clone(),
        attempts: item.attempts.to_string(),
        expires_at: item.expires_at.to_string(),
        fingerprint_text: text::truncate(&item.fingerprint, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_query_slot_views(items: &[ManualQuerySlot]) -> Vec<ManualQuerySlotView> {
    let mut views: Vec<ManualQuerySlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_query_slot_view(item));
    }
    views
}

pub fn sum_manual_query_slot_view(view: &ManualQuerySlotView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_query_slot("code-15", "title-452");
        let view = to_manual_query_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_query_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
