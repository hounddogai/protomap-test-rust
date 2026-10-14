use super::*;

use acme_kit::text;

/// How screens and exports show a default overage reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservationView {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub timezone: String,
    pub host_text: String,
    pub ratio: String,
    pub summary: String,
    pub rate: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_default_overage_reservation_view(item: &DefaultOverageReservation) -> DefaultOverageReservationView {
    DefaultOverageReservationView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        currency: text::truncate(&item.currency, 40),
        timezone: text::truncate(&item.timezone, 80),
        host_text: item.host.clone(),
        ratio: format!("{:.2}", item.ratio),
        summary: item.summary.clone(),
        rate: format!("{:.2}", item.rate),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_overage_reservation_views(items: &[DefaultOverageReservation]) -> Vec<DefaultOverageReservationView> {
    let mut views: Vec<DefaultOverageReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_overage_reservation_view(item));
    }
    views
}

pub fn sum_default_overage_reservation_view(view: &DefaultOverageReservationView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_overage_reservation("code-964", "title-83");
        let view = to_default_overage_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_overage_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
