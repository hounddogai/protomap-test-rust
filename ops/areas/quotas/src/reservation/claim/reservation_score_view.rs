use super::*;

use acme_kit::text;

/// How screens and exports show a reservation score.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScoreView {
    pub code_text: String,
    pub label_text: String,
    pub cluster: String,
    pub ratio: String,
    pub pinned: String,
    pub sequence: String,
    pub name: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_reservation_score_view(item: &ReservationScore) -> ReservationScoreView {
    ReservationScoreView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        cluster: item.cluster.clone(),
        ratio: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        sequence: item.sequence.to_string(),
        name: item.name.clone(),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_score_views(items: &[ReservationScore]) -> Vec<ReservationScoreView> {
    let mut views: Vec<ReservationScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_score_view(item));
    }
    views
}

pub fn sum_reservation_score_view(view: &ReservationScoreView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ratio + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_score("code-56", "label-42");
        let view = to_reservation_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
