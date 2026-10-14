use super::*;

use acme_kit::text;

/// How screens and exports show a global reservation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationVarianceView {
    pub code_text: String,
    pub title_text: String,
    pub batch_size: String,
    pub created_at: String,
    pub ttl_seconds: String,
    pub starts_at: String,
    pub version: String,
    pub weight_text: String,
    pub quantity: String,
    pub amount: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_global_reservation_variance_view(item: &GlobalReservationVariance) -> GlobalReservationVarianceView {
    GlobalReservationVarianceView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        created_at: item.created_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        starts_at: item.starts_at.to_string(),
        version: item.version.to_string(),
        weight_text: format!("{:.2}", item.weight),
        quantity: item.quantity.to_string(),
        amount: format!("{:.2}", item.amount),
        locale: text::truncate(&item.locale, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_reservation_variance_views(items: &[GlobalReservationVariance]) -> Vec<GlobalReservationVarianceView> {
    let mut views: Vec<GlobalReservationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_reservation_variance_view(item));
    }
    views
}

pub fn sum_global_reservation_variance_view(view: &GlobalReservationVarianceView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_created_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_reservation_variance("code-388", "title-360");
        let view = to_global_reservation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_reservation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
