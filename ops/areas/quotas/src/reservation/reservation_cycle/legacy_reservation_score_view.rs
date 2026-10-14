use super::*;

use acme_kit::text;

/// How screens and exports show a legacy reservation score.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScoreView {
    pub code: String,
    pub title_text: String,
    pub external_ref: String,
    pub contact_email: String,
    pub limit: String,
    pub batch_size: String,
    pub weight_text: String,
    pub grace_days_text: String,
    pub issuer_text: String,
    pub name_text: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_legacy_reservation_score_view(item: &LegacyReservationScore) -> LegacyReservationScoreView {
    LegacyReservationScoreView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        external_ref: text::truncate(&item.external_ref, 80),
        contact_email: item.contact_email.clone(),
        limit: item.limit.to_string(),
        batch_size: item.batch_size.to_string(),
        weight_text: format!("{:.2}", item.weight),
        grace_days_text: item.grace_days.to_string(),
        issuer_text: text::truncate(&item.issuer, 64),
        name_text: item.name.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_reservation_score_views(items: &[LegacyReservationScore]) -> Vec<LegacyReservationScoreView> {
    let mut views: Vec<LegacyReservationScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_reservation_score_view(item));
    }
    views
}

pub fn sum_legacy_reservation_score_view(view: &LegacyReservationScoreView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_batch_size as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_reservation_score("code-309", "title-786");
        let view = to_legacy_reservation_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_reservation_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
