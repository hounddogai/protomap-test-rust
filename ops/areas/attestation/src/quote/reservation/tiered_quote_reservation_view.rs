use super::*;

/// How screens and exports show a tiered quote reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReservationView {
    pub code_text: String,
    pub name_text: String,
    pub fingerprint: String,
    pub cipher_text: String,
    pub currency: String,
    pub visible_text: String,
    pub source: String,
    pub ceiling: String,
    pub window_days: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_tiered_quote_reservation_view(item: &TieredQuoteReservation) -> TieredQuoteReservationView {
    TieredQuoteReservationView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        cipher_text: item.cipher.clone(),
        currency: item.currency.clone(),
        visible_text: format!("{}", item.visible),
        source: item.source.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        window_days: item.window_days.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_quote_reservation_views(items: &[TieredQuoteReservation]) -> Vec<TieredQuoteReservationView> {
    let mut views: Vec<TieredQuoteReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_quote_reservation_view(item));
    }
    views
}

pub fn sum_tiered_quote_reservation_view(view: &TieredQuoteReservationView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ceiling + (parsed_window_days as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_quote_reservation("code-405", "name-525");
        let view = to_tiered_quote_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_quote_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
