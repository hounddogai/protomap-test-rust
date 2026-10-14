use super::*;

/// How screens and exports show a seasonal measurement allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementAllocationView {
    pub code_text: String,
    pub label_text: String,
    pub ceiling_text: String,
    pub visible_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_seasonal_measurement_allocation_view(
    item: &SeasonalMeasurementAllocation,
) -> SeasonalMeasurementAllocationView {
    SeasonalMeasurementAllocationView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        visible_text: format!("{}", item.visible),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_measurement_allocation_views(
    items: &[SeasonalMeasurementAllocation],
) -> Vec<SeasonalMeasurementAllocationView> {
    let mut views: Vec<SeasonalMeasurementAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_measurement_allocation_view(item));
    }
    views
}

pub fn sum_seasonal_measurement_allocation_view(view: &SeasonalMeasurementAllocationView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_measurement_allocation("code-479", "label-346");
        let view = to_seasonal_measurement_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_measurement_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
