use super::*;

use acme_kit::text;

/// How screens and exports show a reservation item.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItemView {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub updated_at: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_reservation_item_view(item: &ReservationItem) -> ReservationItemView {
    ReservationItemView {
        code: item.code.clone(),
        title: item.title.clone(),
        notes: text::truncate(&item.notes, 40),
        updated_at: item.updated_at.to_string(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_item_views(items: &[ReservationItem]) -> Vec<ReservationItemView> {
    let mut views: Vec<ReservationItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_item_view(item));
    }
    views
}

pub fn sum_reservation_item_view(view: &ReservationItemView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_item("code-120", "title-591");
        let view = to_reservation_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
