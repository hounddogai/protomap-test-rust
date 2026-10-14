use super::*;

use acme_kit::text;

/// How screens and exports show a query reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryReservationView {
    pub code_text: String,
    pub label: String,
    pub audience: String,
    pub external_ref_text: String,
    pub description: String,
    pub score: String,
    pub weight: String,
    pub rotation_days: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_query_reservation_view(item: &QueryReservation) -> QueryReservationView {
    QueryReservationView {
        code_text: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 64),
        audience: item.audience.clone(),
        external_ref_text: item.external_ref.clone(),
        description: item.description.clone(),
        score: format!("{:.2}", item.score),
        weight: format!("{:.2}", item.weight),
        rotation_days: item.rotation_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_reservation_views(items: &[QueryReservation]) -> Vec<QueryReservationView> {
    let mut views: Vec<QueryReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_reservation_view(item));
    }
    views
}

pub fn sum_query_reservation_view(view: &QueryReservationView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_score + parsed_weight + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_reservation("code-580", "label-961");
        let view = to_query_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
