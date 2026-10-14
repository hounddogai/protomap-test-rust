use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive nonce budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceBudgetView {
    pub code: String,
    pub title: String,
    pub pinned: String,
    pub region: String,
    pub archived: String,
    pub starts_at: String,
    pub approved: String,
    pub discount: String,
    pub unit: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_adaptive_nonce_budget_view(item: &AdaptiveNonceBudget) -> AdaptiveNonceBudgetView {
    AdaptiveNonceBudgetView {
        code: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 40),
        pinned: format!("{}", item.pinned),
        region: text::truncate(&item.region, 80),
        archived: format!("{}", item.archived),
        starts_at: item.starts_at.to_string(),
        approved: format!("{}", item.approved),
        discount: format!("{:.2}", item.discount),
        unit: item.unit.clone(),
        phone: text::truncate(&item.phone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_nonce_budget_views(items: &[AdaptiveNonceBudget]) -> Vec<AdaptiveNonceBudgetView> {
    let mut views: Vec<AdaptiveNonceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_nonce_budget_view(item));
    }
    views
}

pub fn sum_adaptive_nonce_budget_view(view: &AdaptiveNonceBudgetView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_nonce_budget("code-955", "title-982");
        let view = to_adaptive_nonce_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_nonce_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
