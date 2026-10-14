use super::*;

use acme_kit::text;

/// How screens and exports show a page threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct PageThresholdView {
    pub code: String,
    pub label_text: String,
    pub ends_at: String,
    pub attempts: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_page_threshold_view(item: &PageThreshold) -> PageThresholdView {
    PageThresholdView {
        code: text::truncate(&item.code, 64),
        label_text: text::truncate(&item.label, 64),
        ends_at: item.ends_at.to_string(),
        attempts: item.attempts.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_threshold_views(items: &[PageThreshold]) -> Vec<PageThresholdView> {
    let mut views: Vec<PageThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_threshold_view(item));
    }
    views
}

pub fn sum_page_threshold_view(view: &PageThresholdView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_attempts as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_threshold("code-128", "label-890");
        let view = to_page_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
