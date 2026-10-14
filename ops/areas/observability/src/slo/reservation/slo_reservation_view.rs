use super::*;

use acme_kit::text;

/// How screens and exports show a slo reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SloReservationView {
    pub code_text: String,
    pub label: String,
    pub host_text: String,
    pub title: String,
    pub rotation_days_text: String,
    pub key_id: String,
    pub category: String,
    pub version: String,
    pub archived_text: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_slo_reservation_view(item: &SloReservation) -> SloReservationView {
    SloReservationView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        host_text: item.host.clone(),
        title: item.title.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        key_id: item.key_id.clone(),
        category: text::truncate(&item.category, 80),
        version: item.version.to_string(),
        archived_text: format!("{}", item.archived),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_reservation_views(items: &[SloReservation]) -> Vec<SloReservationView> {
    let mut views: Vec<SloReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_reservation_view(item));
    }
    views
}

pub fn sum_slo_reservation_view(view: &SloReservationView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_reservation("code-911", "label-673");
        let view = to_slo_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
