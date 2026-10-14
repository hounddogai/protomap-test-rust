use super::*;

use acme_kit::text;

/// How screens and exports show a capped overage reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub currency: String,
    pub cluster_text: String,
    pub weight_text: String,
    pub cost: String,
    pub category: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_capped_overage_reservation_view(item: &CappedOverageReservation) -> CappedOverageReservationView {
    CappedOverageReservationView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        currency: item.currency.clone(),
        cluster_text: text::truncate(&item.cluster, 40),
        weight_text: format!("{:.2}", item.weight),
        cost: format!("{:.2}", item.cost),
        category: item.category.clone(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_overage_reservation_views(items: &[CappedOverageReservation]) -> Vec<CappedOverageReservationView> {
    let mut views: Vec<CappedOverageReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_overage_reservation_view(item));
    }
    views
}

pub fn sum_capped_overage_reservation_view(view: &CappedOverageReservationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_priority as f64 + parsed_weight + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_overage_reservation("code-282", "title-491");
        let view = to_capped_overage_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_overage_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
