use super::*;

use acme_kit::text;

/// How screens and exports show a probe reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReservationView {
    pub code: String,
    pub label_text: String,
    pub version: String,
    pub batch_size_text: String,
    pub country: String,
    pub notes: String,
    pub ends_at: String,
    pub summary: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_probe_reservation_view(item: &ProbeReservation) -> ProbeReservationView {
    ProbeReservationView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        version: item.version.to_string(),
        batch_size_text: item.batch_size.to_string(),
        country: item.country.clone(),
        notes: item.notes.clone(),
        ends_at: item.ends_at.to_string(),
        summary: item.summary.clone(),
        audience: text::truncate(&item.audience, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_reservation_views(items: &[ProbeReservation]) -> Vec<ProbeReservationView> {
    let mut views: Vec<ProbeReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_reservation_view(item));
    }
    views
}

pub fn sum_probe_reservation_view(view: &ProbeReservationView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_batch_size as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_reservation("code-401", "label-55");
        let view = to_probe_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
