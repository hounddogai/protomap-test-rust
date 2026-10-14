use super::*;

use acme_kit::text;

/// How screens and exports show a runbook slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSlotView {
    pub code_text: String,
    pub label_text: String,
    pub country_text: String,
    pub margin: String,
    pub checksum_text: String,
    pub audience: String,
    pub locale_text: String,
    pub tenant_id: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_runbook_slot_view(item: &RunbookSlot) -> RunbookSlotView {
    RunbookSlotView {
        code_text: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 80),
        country_text: item.country.clone(),
        margin: format!("{:.2}", item.margin),
        checksum_text: item.checksum.clone(),
        audience: item.audience.clone(),
        locale_text: item.locale.clone(),
        tenant_id: item.tenant_id.clone(),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_slot_views(items: &[RunbookSlot]) -> Vec<RunbookSlotView> {
    let mut views: Vec<RunbookSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_slot_view(item));
    }
    views
}

pub fn sum_runbook_slot_view(view: &RunbookSlotView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_slot("code-139", "label-859");
        let view = to_runbook_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
