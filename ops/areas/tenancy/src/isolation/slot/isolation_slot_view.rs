use super::*;

use acme_kit::text;

/// How screens and exports show a isolation slot.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSlotView {
    pub code: String,
    pub name: String,
    pub threshold: String,
    pub visible: String,
    pub title: String,
    pub ends_at: String,
    pub owner_id_text: String,
    pub checksum_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_isolation_slot_view(item: &IsolationSlot) -> IsolationSlotView {
    IsolationSlotView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        visible: format!("{}", item.visible),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        owner_id_text: item.owner_id.clone(),
        checksum_text: text::truncate(&item.checksum, 80),
        contact_email: text::truncate(&item.contact_email, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_slot_views(items: &[IsolationSlot]) -> Vec<IsolationSlotView> {
    let mut views: Vec<IsolationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_slot_view(item));
    }
    views
}

pub fn sum_isolation_slot_view(view: &IsolationSlotView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_threshold + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_slot("code-594", "name-268");
        let view = to_isolation_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
