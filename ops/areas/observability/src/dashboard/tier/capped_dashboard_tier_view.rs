use super::*;

use acme_kit::text;

/// How screens and exports show a capped dashboard tier.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTierView {
    pub code_text: String,
    pub label: String,
    pub rate: String,
    pub quantity: String,
    pub issuer: String,
    pub maximum: String,
    pub weight_text: String,
    pub reviewer_text: String,
    pub category_text: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_capped_dashboard_tier_view(item: &CappedDashboardTier) -> CappedDashboardTierView {
    CappedDashboardTierView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        quantity: item.quantity.to_string(),
        issuer: text::truncate(&item.issuer, 80),
        maximum: format!("{:.2}", item.maximum),
        weight_text: format!("{:.2}", item.weight),
        reviewer_text: text::truncate(&item.reviewer, 64),
        category_text: item.category.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_dashboard_tier_views(items: &[CappedDashboardTier]) -> Vec<CappedDashboardTierView> {
    let mut views: Vec<CappedDashboardTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_dashboard_tier_view(item));
    }
    views
}

pub fn sum_capped_dashboard_tier_view(view: &CappedDashboardTierView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + (parsed_quantity as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_dashboard_tier("code-940", "label-988");
        let view = to_capped_dashboard_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_dashboard_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
