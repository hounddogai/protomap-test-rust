use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard profile.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfileView {
    pub code: String,
    pub label_text: String,
    pub segment_text: String,
    pub external_ref: String,
    pub threshold: String,
    pub contact_email: String,
    pub scope: String,
    pub cipher: String,
    pub name_text: String,
    pub version: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_dashboard_profile_view(item: &DashboardProfile) -> DashboardProfileView {
    DashboardProfileView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        segment_text: text::truncate(&item.segment, 64),
        external_ref: item.external_ref.clone(),
        threshold: format!("{:.2}", item.threshold),
        contact_email: item.contact_email.clone(),
        scope: item.scope.clone(),
        cipher: text::truncate(&item.cipher, 64),
        name_text: text::truncate(&item.name, 64),
        version: item.version.to_string(),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_profile_views(items: &[DashboardProfile]) -> Vec<DashboardProfileView> {
    let mut views: Vec<DashboardProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_profile_view(item));
    }
    views
}

pub fn sum_dashboard_profile_view(view: &DashboardProfileView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_threshold + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_profile("code-960", "label-693");
        let view = to_dashboard_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
