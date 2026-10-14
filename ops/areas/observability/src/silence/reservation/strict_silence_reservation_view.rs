use super::*;

use acme_kit::text;

/// How screens and exports show a strict silence reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilenceReservationView {
    pub code_text: String,
    pub title: String,
    pub region: String,
    pub checksum: String,
    pub source_text: String,
    pub capacity_text: String,
    pub port_text: String,
    pub retries: String,
    pub version: String,
    pub summary_text: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_strict_silence_reservation_view(item: &StrictSilenceReservation) -> StrictSilenceReservationView {
    StrictSilenceReservationView {
        code_text: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 120),
        region: item.region.clone(),
        checksum: text::truncate(&item.checksum, 40),
        source_text: item.source.clone(),
        capacity_text: item.capacity.to_string(),
        port_text: item.port.to_string(),
        retries: item.retries.to_string(),
        version: item.version.to_string(),
        summary_text: item.summary.clone(),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_silence_reservation_views(items: &[StrictSilenceReservation]) -> Vec<StrictSilenceReservationView> {
    let mut views: Vec<StrictSilenceReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_silence_reservation_view(item));
    }
    views
}

pub fn sum_strict_silence_reservation_view(view: &StrictSilenceReservationView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_port as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_silence_reservation("code-95", "title-521");
        let view = to_strict_silence_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_silence_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
