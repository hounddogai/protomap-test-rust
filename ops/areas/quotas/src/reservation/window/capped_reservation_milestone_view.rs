use super::*;

use acme_kit::text;

/// How screens and exports show a capped reservation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneView {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub grace_days: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_capped_reservation_milestone_view(item: &CappedReservationMilestone) -> CappedReservationMilestoneView {
    CappedReservationMilestoneView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        cipher: text::truncate(&item.cipher, 80),
        grace_days: item.grace_days.to_string(),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_reservation_milestone_views(
    items: &[CappedReservationMilestone],
) -> Vec<CappedReservationMilestoneView> {
    let mut views: Vec<CappedReservationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_reservation_milestone_view(item));
    }
    views
}

pub fn sum_capped_reservation_milestone_view(view: &CappedReservationMilestoneView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_reservation_milestone("code-207", "name-220");
        let view = to_capped_reservation_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_reservation_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
