use super::*;

use acme_kit::text;

/// How screens and exports show a regional data export reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationView {
    pub code: String,
    pub label: String,
    pub name: String,
    pub capacity_text: String,
    pub contact_email: String,
    pub owner_id: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_regional_data_export_reservation_view(
    item: &RegionalDataExportReservation,
) -> RegionalDataExportReservationView {
    RegionalDataExportReservationView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 64),
        name: text::truncate(&item.name, 80),
        capacity_text: item.capacity.to_string(),
        contact_email: item.contact_email.clone(),
        owner_id: item.owner_id.clone(),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_data_export_reservation_views(
    items: &[RegionalDataExportReservation],
) -> Vec<RegionalDataExportReservationView> {
    let mut views: Vec<RegionalDataExportReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_data_export_reservation_view(item));
    }
    views
}

pub fn sum_regional_data_export_reservation_view(view: &RegionalDataExportReservationView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_data_export_reservation("code-441", "label-818");
        let view = to_regional_data_export_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_data_export_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
