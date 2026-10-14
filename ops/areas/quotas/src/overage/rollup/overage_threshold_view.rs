use super::*;

use acme_kit::text;

/// How screens and exports show a overage threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdView {
    pub code: String,
    pub name: String,
    pub quantity_text: String,
    pub score_text: String,
    pub budget: String,
    pub key_id: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_overage_threshold_view(item: &OverageThreshold) -> OverageThresholdView {
    OverageThresholdView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 80),
        quantity_text: item.quantity.to_string(),
        score_text: format!("{:.2}", item.score),
        budget: format!("{:.2}", item.budget),
        key_id: text::truncate(&item.key_id, 40),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_threshold_views(items: &[OverageThreshold]) -> Vec<OverageThresholdView> {
    let mut views: Vec<OverageThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_threshold_view(item));
    }
    views
}

pub fn sum_overage_threshold_view(view: &OverageThresholdView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_score + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_threshold("code-240", "name-319");
        let view = to_overage_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
