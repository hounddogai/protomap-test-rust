use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled query estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimateView {
    pub code: String,
    pub title_text: String,
    pub window_days: String,
    pub region: String,
    pub cluster: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_scheduled_query_estimate_view(item: &ScheduledQueryEstimate) -> ScheduledQueryEstimateView {
    ScheduledQueryEstimateView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        window_days: item.window_days.to_string(),
        region: item.region.clone(),
        cluster: item.cluster.clone(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_query_estimate_views(items: &[ScheduledQueryEstimate]) -> Vec<ScheduledQueryEstimateView> {
    let mut views: Vec<ScheduledQueryEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_query_estimate_view(item));
    }
    views
}

pub fn sum_scheduled_query_estimate_view(view: &ScheduledQueryEstimateView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_query_estimate("code-664", "title-644");
        let view = to_scheduled_query_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_query_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
