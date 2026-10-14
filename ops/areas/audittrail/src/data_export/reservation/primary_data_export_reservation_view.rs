use super::*;

use acme_kit::text;

/// How screens and exports show a primary data export reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportReservationView {
    pub code_text: String,
    pub title_text: String,
    pub principal: String,
    pub source_text: String,
    pub revision: String,
    pub ttl_seconds_text: String,
    pub summary: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_primary_data_export_reservation_view(
    item: &PrimaryDataExportReservation,
) -> PrimaryDataExportReservationView {
    PrimaryDataExportReservationView {
        code_text: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        principal: text::truncate(&item.principal, 40),
        source_text: text::truncate(&item.source, 64),
        revision: item.revision.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        summary: text::truncate(&item.summary, 80),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_data_export_reservation_views(
    items: &[PrimaryDataExportReservation],
) -> Vec<PrimaryDataExportReservationView> {
    let mut views: Vec<PrimaryDataExportReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_data_export_reservation_view(item));
    }
    views
}

pub fn sum_primary_data_export_reservation_view(view: &PrimaryDataExportReservationView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_ttl_seconds as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_data_export_reservation("code-264", "title-400");
        let view = to_primary_data_export_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_data_export_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
