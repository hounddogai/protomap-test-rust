use super::*;

use acme_kit::text;

/// How screens and exports show a measurement preference.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreferenceView {
    pub code: String,
    pub title: String,
    pub minimum_text: String,
    pub batch_size: String,
    pub phone_text: String,
    pub description_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_measurement_preference_view(item: &MeasurementPreference) -> MeasurementPreferenceView {
    MeasurementPreferenceView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        batch_size: item.batch_size.to_string(),
        phone_text: item.phone.clone(),
        description_text: item.description.clone(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_preference_views(items: &[MeasurementPreference]) -> Vec<MeasurementPreferenceView> {
    let mut views: Vec<MeasurementPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_preference_view(item));
    }
    views
}

pub fn sum_measurement_preference_view(view: &MeasurementPreferenceView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_minimum + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_preference("code-378", "title-222");
        let view = to_measurement_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
