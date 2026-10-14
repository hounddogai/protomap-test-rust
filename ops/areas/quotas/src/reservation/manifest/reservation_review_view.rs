use super::*;

use acme_kit::text;

/// How screens and exports show a reservation review.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationReviewView {
    pub code: String,
    pub name_text: String,
    pub region_text: String,
    pub discount: String,
    pub created_at: String,
    pub updated_at_text: String,
    pub owner_id_text: String,
    pub score: String,
    pub burst_text: String,
    pub fingerprint_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_reservation_review_view(item: &ReservationReview) -> ReservationReviewView {
    ReservationReviewView {
        code: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 80),
        region_text: item.region.clone(),
        discount: format!("{:.2}", item.discount),
        created_at: item.created_at.to_string(),
        updated_at_text: item.updated_at.to_string(),
        owner_id_text: text::truncate(&item.owner_id, 80),
        score: format!("{:.2}", item.score),
        burst_text: item.burst.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_review_views(items: &[ReservationReview]) -> Vec<ReservationReviewView> {
    let mut views: Vec<ReservationReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_review_view(item));
    }
    views
}

pub fn sum_reservation_review_view(view: &ReservationReviewView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_discount + (parsed_created_at as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_review("code-324", "name-448");
        let view = to_reservation_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
