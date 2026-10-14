use super::*;

use acme_kit::text;

/// How screens and exports show a reservation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationClaimView {
    pub code: String,
    pub label_text: String,
    pub updated_at: String,
    pub retries: String,
    pub window_days: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_reservation_claim_view(item: &ReservationClaim) -> ReservationClaimView {
    ReservationClaimView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        updated_at: item.updated_at.to_string(),
        retries: item.retries.to_string(),
        window_days: item.window_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_claim_views(items: &[ReservationClaim]) -> Vec<ReservationClaimView> {
    let mut views: Vec<ReservationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_claim_view(item));
    }
    views
}

pub fn sum_reservation_claim_view(view: &ReservationClaimView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_retries as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_claim("code-342", "label-767");
        let view = to_reservation_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
