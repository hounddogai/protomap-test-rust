use super::*;

use acme_kit::text;

/// How screens and exports show a isolation threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationThresholdView {
    pub code_text: String,
    pub name: String,
    pub cost: String,
    pub region: String,
    pub host: String,
    pub threshold: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_isolation_threshold_view(item: &IsolationThreshold) -> IsolationThresholdView {
    IsolationThresholdView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        region: text::truncate(&item.region, 40),
        host: item.host.clone(),
        threshold: format!("{:.2}", item.threshold),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_threshold_views(items: &[IsolationThreshold]) -> Vec<IsolationThresholdView> {
    let mut views: Vec<IsolationThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_threshold_view(item));
    }
    views
}

pub fn sum_isolation_threshold_view(view: &IsolationThresholdView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_cost + parsed_threshold + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_threshold("code-786", "name-976");
        let view = to_isolation_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
