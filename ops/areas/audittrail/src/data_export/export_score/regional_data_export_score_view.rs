use super::*;

use acme_kit::text;

/// How screens and exports show a regional data export score.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportScoreView {
    pub code_text: String,
    pub label: String,
    pub priority_text: String,
    pub category: String,
    pub timezone: String,
    pub maximum: String,
    pub version: String,
    pub audience: String,
    pub tenant_id: String,
    pub host_text: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_regional_data_export_score_view(item: &RegionalDataExportScore) -> RegionalDataExportScoreView {
    RegionalDataExportScoreView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        priority_text: item.priority.to_string(),
        category: text::truncate(&item.category, 64),
        timezone: item.timezone.clone(),
        maximum: format!("{:.2}", item.maximum),
        version: item.version.to_string(),
        audience: text::truncate(&item.audience, 120),
        tenant_id: item.tenant_id.clone(),
        host_text: item.host.clone(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_data_export_score_views(items: &[RegionalDataExportScore]) -> Vec<RegionalDataExportScoreView> {
    let mut views: Vec<RegionalDataExportScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_data_export_score_view(item));
    }
    views
}

pub fn sum_regional_data_export_score_view(view: &RegionalDataExportScoreView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_priority as f64 + parsed_maximum + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_data_export_score("code-846", "label-806");
        let view = to_regional_data_export_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_data_export_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
