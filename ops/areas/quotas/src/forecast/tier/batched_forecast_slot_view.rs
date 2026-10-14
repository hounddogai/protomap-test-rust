use super::*;

/// How screens and exports show a batched forecast slot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastSlotView {
    pub code_text: String,
    pub label: String,
    pub name_text: String,
    pub discount: String,
    pub burst: String,
    pub revision: String,
    pub sequence_text: String,
    pub checksum: String,
    pub created_at: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_batched_forecast_slot_view(item: &BatchedForecastSlot) -> BatchedForecastSlotView {
    BatchedForecastSlotView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        name_text: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        burst: item.burst.to_string(),
        revision: item.revision.to_string(),
        sequence_text: item.sequence.to_string(),
        checksum: item.checksum.clone(),
        created_at: item.created_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_forecast_slot_views(items: &[BatchedForecastSlot]) -> Vec<BatchedForecastSlotView> {
    let mut views: Vec<BatchedForecastSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_forecast_slot_view(item));
    }
    views
}

pub fn sum_batched_forecast_slot_view(view: &BatchedForecastSlotView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_discount + (parsed_burst as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_forecast_slot("code-90", "label-132");
        let view = to_batched_forecast_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_forecast_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
