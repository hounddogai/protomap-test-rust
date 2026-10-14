use super::*;

/// How screens and exports show a weighted forecast reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastReservationView {
    pub code: String,
    pub label: String,
    pub ttl_seconds: String,
    pub revision: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_weighted_forecast_reservation_view(item: &WeightedForecastReservation) -> WeightedForecastReservationView {
    WeightedForecastReservationView {
        code: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        revision: item.revision.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_forecast_reservation_views(
    items: &[WeightedForecastReservation],
) -> Vec<WeightedForecastReservationView> {
    let mut views: Vec<WeightedForecastReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_forecast_reservation_view(item));
    }
    views
}

pub fn sum_weighted_forecast_reservation_view(view: &WeightedForecastReservationView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_revision as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_forecast_reservation("code-902", "label-711");
        let view = to_weighted_forecast_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_forecast_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
