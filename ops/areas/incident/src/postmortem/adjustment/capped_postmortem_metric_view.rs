use super::*;

use acme_kit::text;

/// How screens and exports show a capped postmortem metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemMetricView {
    pub code_text: String,
    pub name: String,
    pub phone: String,
    pub audience: String,
    pub category: String,
    pub region: String,
    pub revision_text: String,
    pub window_days: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_capped_postmortem_metric_view(item: &CappedPostmortemMetric) -> CappedPostmortemMetricView {
    CappedPostmortemMetricView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        phone: text::truncate(&item.phone, 120),
        audience: item.audience.clone(),
        category: text::truncate(&item.category, 40),
        region: item.region.clone(),
        revision_text: item.revision.to_string(),
        window_days: item.window_days.to_string(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_postmortem_metric_views(items: &[CappedPostmortemMetric]) -> Vec<CappedPostmortemMetricView> {
    let mut views: Vec<CappedPostmortemMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_postmortem_metric_view(item));
    }
    views
}

pub fn sum_capped_postmortem_metric_view(view: &CappedPostmortemMetricView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_window_days as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_postmortem_metric("code-842", "name-55");
        let view = to_capped_postmortem_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_postmortem_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
