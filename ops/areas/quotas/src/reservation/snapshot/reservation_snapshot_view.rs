use super::*;

use acme_kit::text;

/// How screens and exports show a reservation snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshotView {
    pub code: String,
    pub name_text: String,
    pub external_ref: String,
    pub title: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_reservation_snapshot_view(item: &ReservationSnapshot) -> ReservationSnapshotView {
    ReservationSnapshotView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        external_ref: item.external_ref.clone(),
        title: text::truncate(&item.title, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_snapshot_views(items: &[ReservationSnapshot]) -> Vec<ReservationSnapshotView> {
    let mut views: Vec<ReservationSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_snapshot_view(item));
    }
    views
}

pub fn sum_reservation_snapshot_view(view: &ReservationSnapshotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_snapshot("code-161", "name-478");
        let view = to_reservation_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
