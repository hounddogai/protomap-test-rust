use super::*;

use acme_kit::text;

/// How screens and exports show a weighted failover score.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScoreView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub quantity: String,
    pub country: String,
    pub reviewer: String,
    pub maximum_text: String,
    pub priority: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_weighted_failover_score_view(item: &WeightedFailoverScore) -> WeightedFailoverScoreView {
    WeightedFailoverScoreView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        ratio: format!("{:.2}", item.ratio),
        quantity: item.quantity.to_string(),
        country: item.country.clone(),
        reviewer: item.reviewer.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        priority: item.priority.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_failover_score_views(items: &[WeightedFailoverScore]) -> Vec<WeightedFailoverScoreView> {
    let mut views: Vec<WeightedFailoverScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_failover_score_view(item));
    }
    views
}

pub fn sum_weighted_failover_score_view(view: &WeightedFailoverScoreView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ratio + (parsed_quantity as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_failover_score("code-670", "title-386");
        let view = to_weighted_failover_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_failover_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
