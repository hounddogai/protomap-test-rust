use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal reservation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationProfileView {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub ratio: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_seasonal_reservation_profile_view(item: &SeasonalReservationProfile) -> SeasonalReservationProfileView {
    SeasonalReservationProfileView {
        code: item.code.clone(),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        ratio: format!("{:.2}", item.ratio),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_reservation_profile_views(
    items: &[SeasonalReservationProfile],
) -> Vec<SeasonalReservationProfileView> {
    let mut views: Vec<SeasonalReservationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_reservation_profile_view(item));
    }
    views
}

pub fn sum_seasonal_reservation_profile_view(view: &SeasonalReservationProfileView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_reservation_profile("code-528", "label-143");
        let view = to_seasonal_reservation_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_reservation_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
