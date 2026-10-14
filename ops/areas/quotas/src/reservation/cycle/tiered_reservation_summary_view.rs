use super::*;

use acme_kit::text;

/// How screens and exports show a tiered reservation summary.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationSummaryView {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub cost: String,
    pub issuer: String,
    pub visible: String,
    pub priority: String,
    pub currency: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_tiered_reservation_summary_view(item: &TieredReservationSummary) -> TieredReservationSummaryView {
    TieredReservationSummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        channel: item.channel.clone(),
        cost: format!("{:.2}", item.cost),
        issuer: item.issuer.clone(),
        visible: format!("{}", item.visible),
        priority: item.priority.to_string(),
        currency: text::truncate(&item.currency, 64),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_reservation_summary_views(items: &[TieredReservationSummary]) -> Vec<TieredReservationSummaryView> {
    let mut views: Vec<TieredReservationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_reservation_summary_view(item));
    }
    views
}

pub fn sum_tiered_reservation_summary_view(view: &TieredReservationSummaryView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_cost + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_reservation_summary("code-263", "title-864");
        let view = to_tiered_reservation_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_reservation_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
