use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollback metric.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMetricView {
    pub code: String,
    pub name_text: String,
    pub namespace_text: String,
    pub title: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_legacy_rollback_metric_view(item: &LegacyRollbackMetric) -> LegacyRollbackMetricView {
    LegacyRollbackMetricView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        namespace_text: item.namespace.clone(),
        title: item.title.clone(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollback_metric_views(items: &[LegacyRollbackMetric]) -> Vec<LegacyRollbackMetricView> {
    let mut views: Vec<LegacyRollbackMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollback_metric_view(item));
    }
    views
}

pub fn sum_legacy_rollback_metric_view(view: &LegacyRollbackMetricView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_attempts as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_rollback_metric("code-225", "name-409");
        let view = to_legacy_rollback_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_rollback_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
