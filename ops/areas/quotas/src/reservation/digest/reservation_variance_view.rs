use super::*;

use acme_kit::text;

/// How screens and exports show a reservation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVarianceView {
    pub code: String,
    pub label: String,
    pub limit_text: String,
    pub quota_units: String,
    pub capacity_text: String,
    pub country_text: String,
    pub status_label: String,
}

pub fn to_reservation_variance_view(item: &ReservationVariance) -> ReservationVarianceView {
    ReservationVarianceView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        limit_text: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        capacity_text: item.capacity.to_string(),
        country_text: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_variance_views(items: &[ReservationVariance]) -> Vec<ReservationVarianceView> {
    let mut views: Vec<ReservationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_variance_view(item));
    }
    views
}

pub fn sum_reservation_variance_view(view: &ReservationVarianceView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_quota_units as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_variance("code-375", "label-251");
        let view = to_reservation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
