use super::*;

use acme_kit::text;

/// How screens and exports show a capped reservation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationVarianceView {
    pub code_text: String,
    pub title_text: String,
    pub starts_at: String,
    pub amount: String,
    pub pinned: String,
    pub phone: String,
    pub currency_text: String,
    pub issuer_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_capped_reservation_variance_view(item: &CappedReservationVariance) -> CappedReservationVarianceView {
    CappedReservationVarianceView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        starts_at: item.starts_at.to_string(),
        amount: format!("{:.2}", item.amount),
        pinned: format!("{}", item.pinned),
        phone: text::truncate(&item.phone, 40),
        currency_text: text::truncate(&item.currency, 64),
        issuer_text: item.issuer.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_reservation_variance_views(items: &[CappedReservationVariance]) -> Vec<CappedReservationVarianceView> {
    let mut views: Vec<CappedReservationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_reservation_variance_view(item));
    }
    views
}

pub fn sum_capped_reservation_variance_view(view: &CappedReservationVarianceView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_amount + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_reservation_variance("code-471", "title-141");
        let view = to_capped_reservation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_reservation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
