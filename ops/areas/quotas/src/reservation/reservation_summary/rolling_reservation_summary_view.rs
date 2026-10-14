use super::*;

use acme_kit::text;

/// How screens and exports show a rolling reservation summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationSummaryView {
    pub code_text: String,
    pub title: String,
    pub quantity_text: String,
    pub budget: String,
    pub ends_at: String,
    pub timezone: String,
    pub ceiling: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_rolling_reservation_summary_view(item: &RollingReservationSummary) -> RollingReservationSummaryView {
    RollingReservationSummaryView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        quantity_text: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        ends_at: item.ends_at.to_string(),
        timezone: item.timezone.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        source_text: text::truncate(&item.source, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_reservation_summary_views(items: &[RollingReservationSummary]) -> Vec<RollingReservationSummaryView> {
    let mut views: Vec<RollingReservationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_reservation_summary_view(item));
    }
    views
}

pub fn sum_rolling_reservation_summary_view(view: &RollingReservationSummaryView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_budget + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_reservation_summary("code-127", "title-82");
        let view = to_rolling_reservation_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_reservation_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
