use super::*;

use acme_kit::text;

/// How screens and exports show a identity metric.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMetricView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub rotation_days: String,
    pub country: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_identity_metric_view(item: &IdentityMetric) -> IdentityMetricView {
    IdentityMetricView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        rotation_days: item.rotation_days.to_string(),
        country: item.country.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_metric_views(items: &[IdentityMetric]) -> Vec<IdentityMetricView> {
    let mut views: Vec<IdentityMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_metric_view(item));
    }
    views
}

pub fn sum_identity_metric_view(view: &IdentityMetricView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_metric("code-109", "label-464");
        let view = to_identity_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
