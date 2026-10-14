use super::*;

use acme_kit::text;

/// How screens and exports show a reservation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProfileView {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub external_ref: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_reservation_profile_view(item: &ReservationProfile) -> ReservationProfileView {
    ReservationProfileView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        currency: item.currency.clone(),
        external_ref: item.external_ref.clone(),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_profile_views(items: &[ReservationProfile]) -> Vec<ReservationProfileView> {
    let mut views: Vec<ReservationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_profile_view(item));
    }
    views
}

pub fn sum_reservation_profile_view(view: &ReservationProfileView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_profile("code-676", "title-837");
        let view = to_reservation_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
