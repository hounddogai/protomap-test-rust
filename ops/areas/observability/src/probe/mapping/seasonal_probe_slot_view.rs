use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal probe slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeSlotView {
    pub code_text: String,
    pub label: String,
    pub principal: String,
    pub notes_text: String,
    pub region: String,
    pub reviewer_text: String,
    pub sequence: String,
    pub rotation_days: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_seasonal_probe_slot_view(item: &SeasonalProbeSlot) -> SeasonalProbeSlotView {
    SeasonalProbeSlotView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        principal: item.principal.clone(),
        notes_text: text::truncate(&item.notes, 80),
        region: text::truncate(&item.region, 80),
        reviewer_text: text::truncate(&item.reviewer, 64),
        sequence: item.sequence.to_string(),
        rotation_days: item.rotation_days.to_string(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_probe_slot_views(items: &[SeasonalProbeSlot]) -> Vec<SeasonalProbeSlotView> {
    let mut views: Vec<SeasonalProbeSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_probe_slot_view(item));
    }
    views
}

pub fn sum_seasonal_probe_slot_view(view: &SeasonalProbeSlotView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_probe_slot("code-93", "label-436");
        let view = to_seasonal_probe_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_probe_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
