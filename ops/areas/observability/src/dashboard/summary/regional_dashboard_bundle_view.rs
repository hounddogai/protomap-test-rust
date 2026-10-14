use super::*;

use acme_kit::text;

/// How screens and exports show a regional dashboard bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDashboardBundleView {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub margin: String,
    pub expires_at: String,
    pub issuer: String,
    pub score: String,
    pub channel: String,
    pub timezone_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_regional_dashboard_bundle_view(item: &RegionalDashboardBundle) -> RegionalDashboardBundleView {
    RegionalDashboardBundleView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 64),
        notes: item.notes.clone(),
        margin: format!("{:.2}", item.margin),
        expires_at: item.expires_at.to_string(),
        issuer: item.issuer.clone(),
        score: format!("{:.2}", item.score),
        channel: text::truncate(&item.channel, 80),
        timezone_text: item.timezone.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_dashboard_bundle_views(items: &[RegionalDashboardBundle]) -> Vec<RegionalDashboardBundleView> {
    let mut views: Vec<RegionalDashboardBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_dashboard_bundle_view(item));
    }
    views
}

pub fn sum_regional_dashboard_bundle_view(view: &RegionalDashboardBundleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_margin + (parsed_expires_at as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_dashboard_bundle("code-8", "name-267");
        let view = to_regional_dashboard_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_dashboard_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
