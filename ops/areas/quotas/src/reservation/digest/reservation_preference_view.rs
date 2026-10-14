use super::*;

use acme_kit::text;

/// How screens and exports show a reservation preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreferenceView {
    pub code_text: String,
    pub label_text: String,
    pub summary: String,
    pub scope_text: String,
    pub margin_text: String,
    pub approved: String,
    pub quota_units: String,
    pub channel: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_reservation_preference_view(item: &ReservationPreference) -> ReservationPreferenceView {
    ReservationPreferenceView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        summary: item.summary.clone(),
        scope_text: text::truncate(&item.scope, 40),
        margin_text: format!("{:.2}", item.margin),
        approved: format!("{}", item.approved),
        quota_units: item.quota_units.to_string(),
        channel: item.channel.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_preference_views(items: &[ReservationPreference]) -> Vec<ReservationPreferenceView> {
    let mut views: Vec<ReservationPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_preference_view(item));
    }
    views
}

pub fn sum_reservation_preference_view(view: &ReservationPreferenceView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_margin + (parsed_quota_units as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_preference("code-646", "label-718");
        let view = to_reservation_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
