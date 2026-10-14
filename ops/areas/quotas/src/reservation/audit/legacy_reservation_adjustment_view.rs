use super::*;

use acme_kit::text;

/// How screens and exports show a legacy reservation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationAdjustmentView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub visible: String,
    pub scope: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_legacy_reservation_adjustment_view(item: &LegacyReservationAdjustment) -> LegacyReservationAdjustmentView {
    LegacyReservationAdjustmentView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        visible: format!("{}", item.visible),
        scope: text::truncate(&item.scope, 40),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_reservation_adjustment_views(
    items: &[LegacyReservationAdjustment],
) -> Vec<LegacyReservationAdjustmentView> {
    let mut views: Vec<LegacyReservationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_reservation_adjustment_view(item));
    }
    views
}

pub fn sum_legacy_reservation_adjustment_view(view: &LegacyReservationAdjustmentView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_reservation_adjustment("code-273", "title-202");
        let view = to_legacy_reservation_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_reservation_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
