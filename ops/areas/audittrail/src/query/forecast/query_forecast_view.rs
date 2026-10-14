use super::*;

use acme_kit::text;

/// How screens and exports show a query forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub revision: String,
    pub retries: String,
    pub capacity: String,
    pub country: String,
    pub created_at: String,
    pub audience: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_query_forecast_view(item: &QueryForecast) -> QueryForecastView {
    QueryForecastView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        cipher: item.cipher.clone(),
        revision: item.revision.to_string(),
        retries: item.retries.to_string(),
        capacity: item.capacity.to_string(),
        country: item.country.clone(),
        created_at: item.created_at.to_string(),
        audience: item.audience.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_forecast_views(items: &[QueryForecast]) -> Vec<QueryForecastView> {
    let mut views: Vec<QueryForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_forecast_view(item));
    }
    views
}

pub fn sum_query_forecast_view(view: &QueryForecastView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_retries as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_forecast("code-97", "title-408");
        let view = to_query_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
