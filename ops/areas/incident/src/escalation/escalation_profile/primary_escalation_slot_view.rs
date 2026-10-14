use super::*;

use acme_kit::text;

/// How screens and exports show a primary escalation slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationSlotView {
    pub code: String,
    pub name_text: String,
    pub timezone: String,
    pub threshold: String,
    pub retries: String,
    pub window_days: String,
    pub cluster: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_primary_escalation_slot_view(item: &PrimaryEscalationSlot) -> PrimaryEscalationSlotView {
    PrimaryEscalationSlotView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        timezone: text::truncate(&item.timezone, 64),
        threshold: format!("{:.2}", item.threshold),
        retries: item.retries.to_string(),
        window_days: item.window_days.to_string(),
        cluster: item.cluster.clone(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_escalation_slot_views(items: &[PrimaryEscalationSlot]) -> Vec<PrimaryEscalationSlotView> {
    let mut views: Vec<PrimaryEscalationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_escalation_slot_view(item));
    }
    views
}

pub fn sum_primary_escalation_slot_view(view: &PrimaryEscalationSlotView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_threshold + (parsed_retries as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_escalation_slot("code-339", "name-433");
        let view = to_primary_escalation_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_escalation_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
