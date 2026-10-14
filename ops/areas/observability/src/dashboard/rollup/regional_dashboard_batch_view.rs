use super::*;

use acme_kit::text;

/// How screens and exports show a regional dashboard batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBatchView {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub category: String,
    pub expires_at: String,
    pub summary: String,
    pub starts_at: String,
    pub audience: String,
    pub weight_text: String,
    pub maximum: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_regional_dashboard_batch_view(item: &RegionalDashboardBatch) -> RegionalDashboardBatchView {
    RegionalDashboardBatchView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        key_id: text::truncate(&item.key_id, 120),
        category: item.category.clone(),
        expires_at: item.expires_at.to_string(),
        summary: text::truncate(&item.summary, 40),
        starts_at: item.starts_at.to_string(),
        audience: item.audience.clone(),
        weight_text: format!("{:.2}", item.weight),
        maximum: format!("{:.2}", item.maximum),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_dashboard_batch_views(items: &[RegionalDashboardBatch]) -> Vec<RegionalDashboardBatchView> {
    let mut views: Vec<RegionalDashboardBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_dashboard_batch_view(item));
    }
    views
}

pub fn sum_regional_dashboard_batch_view(view: &RegionalDashboardBatchView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_starts_at as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_dashboard_batch("code-903", "title-104");
        let view = to_regional_dashboard_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_dashboard_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
