use super::*;

use acme_kit::text;

/// How screens and exports show a alert slot.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertSlotView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub tenant_id: String,
    pub revision_text: String,
    pub country: String,
    pub limit: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_alert_slot_view(item: &AlertSlot) -> AlertSlotView {
    AlertSlotView {
        code: item.code.clone(),
        name: item.name.clone(),
        description: text::truncate(&item.description, 64),
        tenant_id: item.tenant_id.clone(),
        revision_text: item.revision.to_string(),
        country: text::truncate(&item.country, 80),
        limit: item.limit.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_slot_views(items: &[AlertSlot]) -> Vec<AlertSlotView> {
    let mut views: Vec<AlertSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_slot_view(item));
    }
    views
}

pub fn sum_alert_slot_view(view: &AlertSlotView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_limit as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_slot("code-763", "name-181");
        let view = to_alert_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
