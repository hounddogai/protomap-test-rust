use super::*;

use acme_kit::text;

/// How screens and exports show a suspension metric.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMetricView {
    pub code_text: String,
    pub title_text: String,
    pub cost: String,
    pub source: String,
    pub budget_text: String,
    pub cluster: String,
    pub description: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_suspension_metric_view(item: &SuspensionMetric) -> SuspensionMetricView {
    SuspensionMetricView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        cost: format!("{:.2}", item.cost),
        source: item.source.clone(),
        budget_text: format!("{:.2}", item.budget),
        cluster: text::truncate(&item.cluster, 120),
        description: item.description.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_metric_views(items: &[SuspensionMetric]) -> Vec<SuspensionMetricView> {
    let mut views: Vec<SuspensionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_metric_view(item));
    }
    views
}

pub fn sum_suspension_metric_view(view: &SuspensionMetricView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_cost + parsed_budget + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_metric("code-231", "title-408");
        let view = to_suspension_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
