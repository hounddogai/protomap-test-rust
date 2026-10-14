use super::*;

use acme_kit::text;

/// How screens and exports show a forecast baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBaselineView {
    pub code: String,
    pub name: String,
    pub label_text: String,
    pub source: String,
    pub issuer_text: String,
    pub contact_email: String,
    pub cost: String,
    pub created_at: String,
    pub approved: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_forecast_baseline_view(item: &ForecastBaseline) -> ForecastBaselineView {
    ForecastBaselineView {
        code: item.code.clone(),
        name: item.name.clone(),
        label_text: item.label.clone(),
        source: item.source.clone(),
        issuer_text: item.issuer.clone(),
        contact_email: item.contact_email.clone(),
        cost: format!("{:.2}", item.cost),
        created_at: item.created_at.to_string(),
        approved: format!("{}", item.approved),
        timezone: text::truncate(&item.timezone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_baseline_views(items: &[ForecastBaseline]) -> Vec<ForecastBaselineView> {
    let mut views: Vec<ForecastBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_baseline_view(item));
    }
    views
}

pub fn sum_forecast_baseline_view(view: &ForecastBaselineView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_cost + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_baseline("code-919", "name-542");
        let view = to_forecast_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
