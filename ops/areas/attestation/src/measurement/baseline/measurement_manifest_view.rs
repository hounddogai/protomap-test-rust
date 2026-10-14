use super::*;

use acme_kit::text;

/// How screens and exports show a measurement manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementManifestView {
    pub code_text: String,
    pub title: String,
    pub unit: String,
    pub audience: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_measurement_manifest_view(item: &MeasurementManifest) -> MeasurementManifestView {
    MeasurementManifestView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        unit: item.unit.clone(),
        audience: item.audience.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_manifest_views(items: &[MeasurementManifest]) -> Vec<MeasurementManifestView> {
    let mut views: Vec<MeasurementManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_manifest_view(item));
    }
    views
}

pub fn sum_measurement_manifest_view(view: &MeasurementManifestView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_measurement_manifest("code-754", "title-466");
        let view = to_measurement_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_measurement_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
