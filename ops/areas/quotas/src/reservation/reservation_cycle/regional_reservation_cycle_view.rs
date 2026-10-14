use super::*;

use acme_kit::text;

/// How screens and exports show a regional reservation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationCycleView {
    pub code_text: String,
    pub label_text: String,
    pub namespace_text: String,
    pub discount_text: String,
    pub description: String,
    pub reviewer: String,
    pub country: String,
    pub expires_at: String,
    pub contact_email: String,
    pub summary: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_regional_reservation_cycle_view(item: &RegionalReservationCycle) -> RegionalReservationCycleView {
    RegionalReservationCycleView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        namespace_text: item.namespace.clone(),
        discount_text: format!("{:.2}", item.discount),
        description: item.description.clone(),
        reviewer: text::truncate(&item.reviewer, 80),
        country: item.country.clone(),
        expires_at: item.expires_at.to_string(),
        contact_email: text::truncate(&item.contact_email, 64),
        summary: item.summary.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_reservation_cycle_views(items: &[RegionalReservationCycle]) -> Vec<RegionalReservationCycleView> {
    let mut views: Vec<RegionalReservationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_reservation_cycle_view(item));
    }
    views
}

pub fn sum_regional_reservation_cycle_view(view: &RegionalReservationCycleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_discount + (parsed_expires_at as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_reservation_cycle("code-406", "label-916");
        let view = to_regional_reservation_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_reservation_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
