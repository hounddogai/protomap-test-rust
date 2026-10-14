use super::*;

use acme_kit::text;

/// How screens and exports show a freeze metric.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMetricView {
    pub code_text: String,
    pub name_text: String,
    pub attempts: String,
    pub fingerprint_text: String,
    pub approved: String,
    pub source: String,
    pub visible: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_freeze_metric_view(item: &FreezeMetric) -> FreezeMetricView {
    FreezeMetricView {
        code_text: text::truncate(&item.code, 40),
        name_text: text::truncate(&item.name, 80),
        attempts: item.attempts.to_string(),
        fingerprint_text: text::truncate(&item.fingerprint, 64),
        approved: format!("{}", item.approved),
        source: text::truncate(&item.source, 120),
        visible: format!("{}", item.visible),
        owner_id: text::truncate(&item.owner_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_metric_views(items: &[FreezeMetric]) -> Vec<FreezeMetricView> {
    let mut views: Vec<FreezeMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_metric_view(item));
    }
    views
}

pub fn sum_freeze_metric_view(view: &FreezeMetricView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
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
        let item = new_freeze_metric("code-701", "name-841");
        let view = to_freeze_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
