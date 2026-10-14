use super::*;

use acme_kit::text;

/// How screens and exports show a token reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservationView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub visible: String,
    pub scope: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_token_reservation_view(item: &TokenReservation) -> TokenReservationView {
    TokenReservationView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        summary: item.summary.clone(),
        visible: format!("{}", item.visible),
        scope: item.scope.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_reservation_views(items: &[TokenReservation]) -> Vec<TokenReservationView> {
    let mut views: Vec<TokenReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_reservation_view(item));
    }
    views
}

pub fn sum_token_reservation_view(view: &TokenReservationView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_reservation("code-369", "title-702");
        let view = to_token_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
