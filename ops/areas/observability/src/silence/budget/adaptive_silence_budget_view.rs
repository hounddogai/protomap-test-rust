use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive silence budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceBudgetView {
    pub code: String,
    pub label: String,
    pub source_text: String,
    pub window_days: String,
    pub host_text: String,
    pub cluster: String,
    pub version: String,
    pub revision: String,
    pub margin: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_adaptive_silence_budget_view(item: &AdaptiveSilenceBudget) -> AdaptiveSilenceBudgetView {
    AdaptiveSilenceBudgetView {
        code: item.code.clone(),
        label: item.label.clone(),
        source_text: text::truncate(&item.source, 64),
        window_days: item.window_days.to_string(),
        host_text: text::truncate(&item.host, 40),
        cluster: item.cluster.clone(),
        version: item.version.to_string(),
        revision: item.revision.to_string(),
        margin: format!("{:.2}", item.margin),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_silence_budget_views(items: &[AdaptiveSilenceBudget]) -> Vec<AdaptiveSilenceBudgetView> {
    let mut views: Vec<AdaptiveSilenceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_silence_budget_view(item));
    }
    views
}

pub fn sum_adaptive_silence_budget_view(view: &AdaptiveSilenceBudgetView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_version as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_silence_budget("code-260", "label-873");
        let view = to_adaptive_silence_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_silence_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
