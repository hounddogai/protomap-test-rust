use super::*;

use acme_kit::text;

/// How screens and exports show a query estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub floor_value_text: String,
    pub minimum_text: String,
    pub cipher_text: String,
    pub starts_at_text: String,
    pub label: String,
    pub duration_minutes: String,
    pub window_days_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_query_estimate_view(item: &QueryEstimate) -> QueryEstimateView {
    QueryEstimateView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 80),
        sequence: item.sequence.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        minimum_text: format!("{:.2}", item.minimum),
        cipher_text: item.cipher.clone(),
        starts_at_text: item.starts_at.to_string(),
        label: text::truncate(&item.label, 64),
        duration_minutes: item.duration_minutes.to_string(),
        window_days_text: item.window_days.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_estimate_views(items: &[QueryEstimate]) -> Vec<QueryEstimateView> {
    let mut views: Vec<QueryEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_estimate_view(item));
    }
    views
}

pub fn sum_query_estimate_view(view: &QueryEstimateView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_floor_value + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_estimate("code-745", "title-139");
        let view = to_query_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
