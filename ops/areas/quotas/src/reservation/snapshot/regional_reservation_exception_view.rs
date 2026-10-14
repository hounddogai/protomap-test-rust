use super::*;

use acme_kit::text;

/// How screens and exports show a regional reservation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationExceptionView {
    pub code: String,
    pub label: String,
    pub cipher_text: String,
    pub title: String,
    pub ttl_seconds_text: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_regional_reservation_exception_view(item: &RegionalReservationException) -> RegionalReservationExceptionView {
    RegionalReservationExceptionView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        cipher_text: item.cipher.clone(),
        title: item.title.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_reservation_exception_views(
    items: &[RegionalReservationException],
) -> Vec<RegionalReservationExceptionView> {
    let mut views: Vec<RegionalReservationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_reservation_exception_view(item));
    }
    views
}

pub fn sum_regional_reservation_exception_view(view: &RegionalReservationExceptionView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_reservation_exception("code-346", "label-899");
        let view = to_regional_reservation_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_reservation_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
