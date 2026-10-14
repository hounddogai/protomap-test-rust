use super::*;

use acme_kit::text;

/// How screens and exports show a probe metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMetricView {
    pub code: String,
    pub title_text: String,
    pub created_at: String,
    pub quota_units_text: String,
    pub notes: String,
    pub cost: String,
    pub margin: String,
    pub reviewer_text: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_probe_metric_view(item: &ProbeMetric) -> ProbeMetricView {
    ProbeMetricView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        created_at: item.created_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        notes: text::truncate(&item.notes, 120),
        cost: format!("{:.2}", item.cost),
        margin: format!("{:.2}", item.margin),
        reviewer_text: text::truncate(&item.reviewer, 80),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_metric_views(items: &[ProbeMetric]) -> Vec<ProbeMetricView> {
    let mut views: Vec<ProbeMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_metric_view(item));
    }
    views
}

pub fn sum_probe_metric_view(view: &ProbeMetricView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quota_units as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_metric("code-104", "title-935");
        let view = to_probe_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
