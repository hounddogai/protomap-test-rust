use super::*;

use acme_kit::text;

/// How screens and exports show a release metric.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetricView {
    pub code_text: String,
    pub label_text: String,
    pub cipher: String,
    pub country: String,
    pub description_text: String,
    pub contact_email: String,
    pub summary_text: String,
    pub name: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_release_metric_view(item: &ReleaseMetric) -> ReleaseMetricView {
    ReleaseMetricView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        cipher: item.cipher.clone(),
        country: item.country.clone(),
        description_text: item.description.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        summary_text: item.summary.clone(),
        name: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_metric_views(items: &[ReleaseMetric]) -> Vec<ReleaseMetricView> {
    let mut views: Vec<ReleaseMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_metric_view(item));
    }
    views
}

pub fn sum_release_metric_view(view: &ReleaseMetricView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_metric("code-985", "label-844");
        let view = to_release_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
