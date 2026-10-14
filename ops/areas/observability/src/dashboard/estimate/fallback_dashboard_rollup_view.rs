use super::*;

use acme_kit::text;

/// How screens and exports show a fallback dashboard rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardRollupView {
    pub code: String,
    pub title: String,
    pub minimum: String,
    pub priority: String,
    pub notes: String,
    pub archived: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_fallback_dashboard_rollup_view(item: &FallbackDashboardRollup) -> FallbackDashboardRollupView {
    FallbackDashboardRollupView {
        code: item.code.clone(),
        title: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        priority: item.priority.to_string(),
        notes: text::truncate(&item.notes, 80),
        archived: format!("{}", item.archived),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_dashboard_rollup_views(items: &[FallbackDashboardRollup]) -> Vec<FallbackDashboardRollupView> {
    let mut views: Vec<FallbackDashboardRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_dashboard_rollup_view(item));
    }
    views
}

pub fn sum_fallback_dashboard_rollup_view(view: &FallbackDashboardRollupView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_minimum + (parsed_priority as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_dashboard_rollup("code-425", "title-420");
        let view = to_fallback_dashboard_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_dashboard_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
