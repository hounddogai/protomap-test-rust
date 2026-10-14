use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyReservationView {
    pub code: String,
    pub title: String,
    pub updated_at: String,
    pub checksum: String,
    pub description: String,
    pub external_ref: String,
    pub issuer_text: String,
    pub currency: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_ttl_policy_reservation_view(item: &TtlPolicyReservation) -> TtlPolicyReservationView {
    TtlPolicyReservationView {
        code: item.code.clone(),
        title: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        checksum: text::truncate(&item.checksum, 120),
        description: text::truncate(&item.description, 64),
        external_ref: item.external_ref.clone(),
        issuer_text: item.issuer.clone(),
        currency: text::truncate(&item.currency, 80),
        summary_text: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_reservation_views(items: &[TtlPolicyReservation]) -> Vec<TtlPolicyReservationView> {
    let mut views: Vec<TtlPolicyReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_reservation_view(item));
    }
    views
}

pub fn sum_ttl_policy_reservation_view(view: &TtlPolicyReservationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_reservation("code-873", "title-622");
        let view = to_ttl_policy_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
