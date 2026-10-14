use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal reservation draft.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationDraftView {
    pub code: String,
    pub title_text: String,
    pub budget: String,
    pub cluster_text: String,
    pub scope: String,
    pub quantity_text: String,
    pub segment: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_seasonal_reservation_draft_view(item: &SeasonalReservationDraft) -> SeasonalReservationDraftView {
    SeasonalReservationDraftView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        cluster_text: text::truncate(&item.cluster, 80),
        scope: item.scope.clone(),
        quantity_text: item.quantity.to_string(),
        segment: item.segment.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_reservation_draft_views(items: &[SeasonalReservationDraft]) -> Vec<SeasonalReservationDraftView> {
    let mut views: Vec<SeasonalReservationDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_reservation_draft_view(item));
    }
    views
}

pub fn sum_seasonal_reservation_draft_view(view: &SeasonalReservationDraftView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_budget + (parsed_quantity as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_reservation_draft("code-65", "title-317");
        let view = to_seasonal_reservation_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_reservation_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
