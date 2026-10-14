use super::*;

/// How screens and exports show a seasonal token reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenReservationView {
    pub code_text: String,
    pub title_text: String,
    pub audience: String,
    pub quota_units: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_seasonal_token_reservation_view(item: &SeasonalTokenReservation) -> SeasonalTokenReservationView {
    SeasonalTokenReservationView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        audience: item.audience.clone(),
        quota_units: item.quota_units.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_token_reservation_views(items: &[SeasonalTokenReservation]) -> Vec<SeasonalTokenReservationView> {
    let mut views: Vec<SeasonalTokenReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_token_reservation_view(item));
    }
    views
}

pub fn sum_seasonal_token_reservation_view(view: &SeasonalTokenReservationView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_token_reservation("code-863", "title-757");
        let view = to_seasonal_token_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_token_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
