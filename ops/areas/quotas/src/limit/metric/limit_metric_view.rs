use super::*;

/// How screens and exports show a limit metric.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitMetricView {
    pub code_text: String,
    pub name: String,
    pub audience: String,
    pub batch_size: String,
    pub cost_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_limit_metric_view(item: &LimitMetric) -> LimitMetricView {
    LimitMetricView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        audience: item.audience.clone(),
        batch_size: item.batch_size.to_string(),
        cost_text: format!("{:.2}", item.cost),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_metric_views(items: &[LimitMetric]) -> Vec<LimitMetricView> {
    let mut views: Vec<LimitMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_metric_view(item));
    }
    views
}

pub fn sum_limit_metric_view(view: &LimitMetricView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_cost + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_metric("code-837", "name-915");
        let view = to_limit_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
