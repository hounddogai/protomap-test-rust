use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive measurement entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementEntryView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub source: String,
    pub checksum: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_adaptive_measurement_entry_view(item: &AdaptiveMeasurementEntry) -> AdaptiveMeasurementEntryView {
    AdaptiveMeasurementEntryView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        source: text::truncate(&item.source, 64),
        checksum: text::truncate(&item.checksum, 40),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_measurement_entry_views(items: &[AdaptiveMeasurementEntry]) -> Vec<AdaptiveMeasurementEntryView> {
    let mut views: Vec<AdaptiveMeasurementEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_measurement_entry_view(item));
    }
    views
}

pub fn sum_adaptive_measurement_entry_view(view: &AdaptiveMeasurementEntryView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_measurement_entry("code-541", "name-910");
        let view = to_adaptive_measurement_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_measurement_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
