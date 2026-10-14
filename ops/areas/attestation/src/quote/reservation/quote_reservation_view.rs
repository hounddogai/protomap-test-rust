use super::*;

use acme_kit::text;

/// How screens and exports show a quote reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReservationView {
    pub code: String,
    pub title_text: String,
    pub rotation_days: String,
    pub rate: String,
    pub burst_text: String,
    pub timezone: String,
    pub amount: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_quote_reservation_view(item: &QuoteReservation) -> QuoteReservationView {
    QuoteReservationView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 64),
        rotation_days: item.rotation_days.to_string(),
        rate: format!("{:.2}", item.rate),
        burst_text: item.burst.to_string(),
        timezone: item.timezone.clone(),
        amount: format!("{:.2}", item.amount),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_reservation_views(items: &[QuoteReservation]) -> Vec<QuoteReservationView> {
    let mut views: Vec<QuoteReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_reservation_view(item));
    }
    views
}

pub fn sum_quote_reservation_view(view: &QuoteReservationView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_rate + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_reservation("code-142", "title-903");
        let view = to_quote_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
