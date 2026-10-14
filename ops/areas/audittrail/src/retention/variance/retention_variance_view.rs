use super::*;

use acme_kit::text;

/// How screens and exports show a retention variance.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionVarianceView {
    pub code: String,
    pub label: String,
    pub rotation_days_text: String,
    pub segment: String,
    pub threshold: String,
    pub starts_at: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_retention_variance_view(item: &RetentionVariance) -> RetentionVarianceView {
    RetentionVarianceView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        rotation_days_text: item.rotation_days.to_string(),
        segment: item.segment.clone(),
        threshold: format!("{:.2}", item.threshold),
        starts_at: item.starts_at.to_string(),
        category: text::truncate(&item.category, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_variance_views(items: &[RetentionVariance]) -> Vec<RetentionVarianceView> {
    let mut views: Vec<RetentionVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_variance_view(item));
    }
    views
}

pub fn sum_retention_variance_view(view: &RetentionVarianceView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_threshold + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_variance("code-239", "label-689");
        let view = to_retention_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
