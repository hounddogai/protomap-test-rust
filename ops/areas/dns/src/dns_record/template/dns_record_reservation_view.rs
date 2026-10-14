use super::*;

use acme_kit::text;

/// How screens and exports show a dns record reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservationView {
    pub code_text: String,
    pub label_text: String,
    pub unit: String,
    pub score: String,
    pub duration_minutes: String,
    pub region_text: String,
    pub expires_at_text: String,
    pub version: String,
    pub threshold: String,
    pub timezone_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_dns_record_reservation_view(item: &DnsRecordReservation) -> DnsRecordReservationView {
    DnsRecordReservationView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        unit: text::truncate(&item.unit, 120),
        score: format!("{:.2}", item.score),
        duration_minutes: item.duration_minutes.to_string(),
        region_text: item.region.clone(),
        expires_at_text: item.expires_at.to_string(),
        version: item.version.to_string(),
        threshold: format!("{:.2}", item.threshold),
        timezone_text: item.timezone.clone(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_reservation_views(items: &[DnsRecordReservation]) -> Vec<DnsRecordReservationView> {
    let mut views: Vec<DnsRecordReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_reservation_view(item));
    }
    views
}

pub fn sum_dns_record_reservation_view(view: &DnsRecordReservationView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_score + (parsed_duration_minutes as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_reservation("code-26", "label-216");
        let view = to_dns_record_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
