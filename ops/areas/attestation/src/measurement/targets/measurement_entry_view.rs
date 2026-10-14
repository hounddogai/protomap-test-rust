use super::*;

/// How screens and exports show a measurement entry.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEntryView {
    pub code: String,
    pub name: String,
    pub active: String,
    pub duration_minutes_text: String,
    pub amount: String,
    pub pinned_text: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_measurement_entry_view(item: &MeasurementEntry) -> MeasurementEntryView {
    MeasurementEntryView {
        code: item.code.clone(),
        name: item.name.clone(),
        active: format!("{}", item.active),
        duration_minutes_text: item.duration_minutes.to_string(),
        amount: format!("{:.2}", item.amount),
        pinned_text: format!("{}", item.pinned),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_entry_views(items: &[MeasurementEntry]) -> Vec<MeasurementEntryView> {
    let mut views: Vec<MeasurementEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_entry_view(item));
    }
    views
}

pub fn sum_measurement_entry_view(view: &MeasurementEntryView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_entry("code-941", "name-829");
        let view = to_measurement_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
