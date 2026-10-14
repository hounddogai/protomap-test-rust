use super::*;

use acme_kit::text;

/// How screens and exports show a peering metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetricView {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub category: String,
    pub source_text: String,
    pub discount: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_peering_metric_view(item: &PeeringMetric) -> PeeringMetricView {
    PeeringMetricView {
        code: item.code.clone(),
        name: item.name.clone(),
        currency: item.currency.clone(),
        category: item.category.clone(),
        source_text: text::truncate(&item.source, 40),
        discount: format!("{:.2}", item.discount),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_metric_views(items: &[PeeringMetric]) -> Vec<PeeringMetricView> {
    let mut views: Vec<PeeringMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_metric_view(item));
    }
    views
}

pub fn sum_peering_metric_view(view: &PeeringMetricView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_discount + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_metric("code-9", "name-578");
        let view = to_peering_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
