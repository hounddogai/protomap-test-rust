use super::*;

use acme_kit::text;

/// How screens and exports show a federation slot.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationSlotView {
    pub code: String,
    pub name: String,
    pub quota_units_text: String,
    pub attempts: String,
    pub expires_at_text: String,
    pub contact_email: String,
    pub notes: String,
    pub sequence: String,
    pub source: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_federation_slot_view(item: &FederationSlot) -> FederationSlotView {
    FederationSlotView {
        code: item.code.clone(),
        name: item.name.clone(),
        quota_units_text: item.quota_units.to_string(),
        attempts: item.attempts.to_string(),
        expires_at_text: item.expires_at.to_string(),
        contact_email: item.contact_email.clone(),
        notes: item.notes.clone(),
        sequence: item.sequence.to_string(),
        source: item.source.clone(),
        audience: text::truncate(&item.audience, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_slot_views(items: &[FederationSlot]) -> Vec<FederationSlotView> {
    let mut views: Vec<FederationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_slot_view(item));
    }
    views
}

pub fn sum_federation_slot_view(view: &FederationSlotView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_attempts as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_slot("code-398", "name-925");
        let view = to_federation_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
