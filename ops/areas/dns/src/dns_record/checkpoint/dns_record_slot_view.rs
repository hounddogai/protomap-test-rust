use super::*;

use acme_kit::text;

/// How screens and exports show a dns record slot.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSlotView {
    pub code: String,
    pub name: String,
    pub source: String,
    pub notes_text: String,
    pub tenant_id: String,
    pub host: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_dns_record_slot_view(item: &DnsRecordSlot) -> DnsRecordSlotView {
    DnsRecordSlotView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        source: item.source.clone(),
        notes_text: item.notes.clone(),
        tenant_id: item.tenant_id.clone(),
        host: item.host.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_slot_views(items: &[DnsRecordSlot]) -> Vec<DnsRecordSlotView> {
    let mut views: Vec<DnsRecordSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_slot_view(item));
    }
    views
}

pub fn sum_dns_record_slot_view(view: &DnsRecordSlotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_slot("code-452", "name-544");
        let view = to_dns_record_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
