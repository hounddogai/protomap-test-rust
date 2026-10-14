use super::*;

use acme_kit::text;

/// How screens and exports show a lenient token reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenReservationView {
    pub code: String,
    pub label_text: String,
    pub name: String,
    pub priority: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_lenient_token_reservation_view(item: &LenientTokenReservation) -> LenientTokenReservationView {
    LenientTokenReservationView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        name: item.name.clone(),
        priority: item.priority.to_string(),
        title: text::truncate(&item.title, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_token_reservation_views(items: &[LenientTokenReservation]) -> Vec<LenientTokenReservationView> {
    let mut views: Vec<LenientTokenReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_token_reservation_view(item));
    }
    views
}

pub fn sum_lenient_token_reservation_view(view: &LenientTokenReservationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_token_reservation("code-234", "label-490");
        let view = to_lenient_token_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_token_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
