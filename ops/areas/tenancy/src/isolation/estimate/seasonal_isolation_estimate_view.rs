use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal isolation estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationEstimateView {
    pub code_text: String,
    pub label_text: String,
    pub window_days: String,
    pub created_at: String,
    pub cost: String,
    pub description: String,
    pub title: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_seasonal_isolation_estimate_view(item: &SeasonalIsolationEstimate) -> SeasonalIsolationEstimateView {
    SeasonalIsolationEstimateView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        window_days: item.window_days.to_string(),
        created_at: item.created_at.to_string(),
        cost: format!("{:.2}", item.cost),
        description: text::truncate(&item.description, 40),
        title: item.title.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_isolation_estimate_views(items: &[SeasonalIsolationEstimate]) -> Vec<SeasonalIsolationEstimateView> {
    let mut views: Vec<SeasonalIsolationEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_isolation_estimate_view(item));
    }
    views
}

pub fn sum_seasonal_isolation_estimate_view(view: &SeasonalIsolationEstimateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_created_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_isolation_estimate("code-685", "label-806");
        let view = to_seasonal_isolation_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_isolation_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
