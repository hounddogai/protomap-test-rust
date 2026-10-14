use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservationView {
    pub code_text: String,
    pub title: String,
    pub checksum: String,
    pub source: String,
    pub contact_email: String,
    pub ends_at: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_trust_domain_reservation_view(item: &TrustDomainReservation) -> TrustDomainReservationView {
    TrustDomainReservationView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        checksum: text::truncate(&item.checksum, 120),
        source: item.source.clone(),
        contact_email: item.contact_email.clone(),
        ends_at: item.ends_at.to_string(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_reservation_views(items: &[TrustDomainReservation]) -> Vec<TrustDomainReservationView> {
    let mut views: Vec<TrustDomainReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_reservation_view(item));
    }
    views
}

pub fn sum_trust_domain_reservation_view(view: &TrustDomainReservationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_reservation("code-285", "title-343");
        let view = to_trust_domain_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
