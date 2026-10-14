use super::*;

use acme_kit::text;

/// How screens and exports show a nonce slot.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlotView {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub reviewer: String,
    pub window_days: String,
    pub retries_text: String,
    pub contact_email_text: String,
    pub version: String,
    pub revision: String,
    pub grace_days_text: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_nonce_slot_view(item: &NonceSlot) -> NonceSlotView {
    NonceSlotView {
        code: item.code.clone(),
        name: item.name.clone(),
        external_ref: item.external_ref.clone(),
        reviewer: item.reviewer.clone(),
        window_days: item.window_days.to_string(),
        retries_text: item.retries.to_string(),
        contact_email_text: text::truncate(&item.contact_email, 120),
        version: item.version.to_string(),
        revision: item.revision.to_string(),
        grace_days_text: item.grace_days.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_slot_views(items: &[NonceSlot]) -> Vec<NonceSlotView> {
    let mut views: Vec<NonceSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_slot_view(item));
    }
    views
}

pub fn sum_nonce_slot_view(view: &NonceSlotView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_retries as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_slot("code-229", "name-425");
        let view = to_nonce_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
