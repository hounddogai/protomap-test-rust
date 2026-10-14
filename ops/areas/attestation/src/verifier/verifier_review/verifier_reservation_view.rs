use super::*;

use acme_kit::text;

/// How screens and exports show a verifier reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationView {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub region_text: String,
    pub channel: String,
    pub minimum_text: String,
    pub score_text: String,
    pub source_text: String,
    pub starts_at_text: String,
    pub created_at: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_verifier_reservation_view(item: &VerifierReservation) -> VerifierReservationView {
    VerifierReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        currency: text::truncate(&item.currency, 120),
        region_text: item.region.clone(),
        channel: item.channel.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        score_text: format!("{:.2}", item.score),
        source_text: item.source.clone(),
        starts_at_text: item.starts_at.to_string(),
        created_at: item.created_at.to_string(),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_reservation_views(items: &[VerifierReservation]) -> Vec<VerifierReservationView> {
    let mut views: Vec<VerifierReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_reservation_view(item));
    }
    views
}

pub fn sum_verifier_reservation_view(view: &VerifierReservationView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_minimum + parsed_score + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_reservation("code-356", "name-999");
        let view = to_verifier_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
