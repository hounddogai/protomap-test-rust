use super::*;

use acme_kit::text;

/// How screens and exports show a measurement slot.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSlotView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub tenant_id: String,
    pub checksum: String,
    pub cost_text: String,
    pub description: String,
    pub batch_size: String,
    pub revision_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_measurement_slot_view(item: &MeasurementSlot) -> MeasurementSlotView {
    MeasurementSlotView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        notes: item.notes.clone(),
        tenant_id: item.tenant_id.clone(),
        checksum: item.checksum.clone(),
        cost_text: format!("{:.2}", item.cost),
        description: item.description.clone(),
        batch_size: item.batch_size.to_string(),
        revision_text: item.revision.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_slot_views(items: &[MeasurementSlot]) -> Vec<MeasurementSlotView> {
    let mut views: Vec<MeasurementSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_slot_view(item));
    }
    views
}

pub fn sum_measurement_slot_view(view: &MeasurementSlotView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_cost + (parsed_batch_size as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_slot("code-446", "name-699");
        let view = to_measurement_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
