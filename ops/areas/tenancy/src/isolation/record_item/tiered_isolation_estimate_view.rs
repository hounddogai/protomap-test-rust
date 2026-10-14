use super::*;

use acme_kit::text;

/// How screens and exports show a tiered isolation estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateView {
    pub code: String,
    pub title: String,
    pub active_text: String,
    pub category: String,
    pub minimum: String,
    pub contact_email: String,
    pub country_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_tiered_isolation_estimate_view(item: &TieredIsolationEstimate) -> TieredIsolationEstimateView {
    TieredIsolationEstimateView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        active_text: format!("{}", item.active),
        category: item.category.clone(),
        minimum: format!("{:.2}", item.minimum),
        contact_email: item.contact_email.clone(),
        country_text: text::truncate(&item.country, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_isolation_estimate_views(items: &[TieredIsolationEstimate]) -> Vec<TieredIsolationEstimateView> {
    let mut views: Vec<TieredIsolationEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_isolation_estimate_view(item));
    }
    views
}

pub fn sum_tiered_isolation_estimate_view(view: &TieredIsolationEstimateView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_minimum + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_isolation_estimate("code-497", "title-281");
        let view = to_tiered_isolation_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_isolation_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
