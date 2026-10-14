use super::*;

/// How screens and exports show a silence reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationView {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub cipher: String,
    pub fingerprint_text: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_silence_reservation_view(item: &SilenceReservation) -> SilenceReservationView {
    SilenceReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        issuer: item.issuer.clone(),
        cipher: item.cipher.clone(),
        fingerprint_text: item.fingerprint.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_reservation_views(items: &[SilenceReservation]) -> Vec<SilenceReservationView> {
    let mut views: Vec<SilenceReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_reservation_view(item));
    }
    views
}

pub fn sum_silence_reservation_view(view: &SilenceReservationView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}
