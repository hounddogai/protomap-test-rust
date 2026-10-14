use super::*;

use acme_kit::text;

/// How screens and exports show a global failover slot.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFailoverSlotView {
    pub code_text: String,
    pub name: String,
    pub category_text: String,
    pub weight: String,
    pub updated_at: String,
    pub contact_email_text: String,
    pub port: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_global_failover_slot_view(item: &GlobalFailoverSlot) -> GlobalFailoverSlotView {
    GlobalFailoverSlotView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        category_text: text::truncate(&item.category, 120),
        weight: format!("{:.2}", item.weight),
        updated_at: item.updated_at.to_string(),
        contact_email_text: item.contact_email.clone(),
        port: item.port.to_string(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_failover_slot_views(items: &[GlobalFailoverSlot]) -> Vec<GlobalFailoverSlotView> {
    let mut views: Vec<GlobalFailoverSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_failover_slot_view(item));
    }
    views
}

pub fn sum_global_failover_slot_view(view: &GlobalFailoverSlotView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_weight + (parsed_updated_at as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_failover_slot("code-4", "name-800");
        let view = to_global_failover_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_failover_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
