use super::*;

use acme_kit::text;

/// How screens and exports show a reservation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationExceptionView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub ratio_text: String,
    pub minimum_text: String,
    pub external_ref: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_reservation_exception_view(item: &ReservationException) -> ReservationExceptionView {
    ReservationExceptionView {
        code: item.code.clone(),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        minimum_text: format!("{:.2}", item.minimum),
        external_ref: item.external_ref.clone(),
        unit: text::truncate(&item.unit, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_exception_views(items: &[ReservationException]) -> Vec<ReservationExceptionView> {
    let mut views: Vec<ReservationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_exception_view(item));
    }
    views
}

pub fn sum_reservation_exception_view(view: &ReservationExceptionView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_ratio + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_exception("code-422", "name-954");
        let view = to_reservation_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
