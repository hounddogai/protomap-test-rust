use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard override.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideView {
    pub code_text: String,
    pub label_text: String,
    pub issuer: String,
    pub version: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_dashboard_override_view(item: &DashboardOverride) -> DashboardOverrideView {
    DashboardOverrideView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        issuer: text::truncate(&item.issuer, 120),
        version: item.version.to_string(),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_override_views(items: &[DashboardOverride]) -> Vec<DashboardOverrideView> {
    let mut views: Vec<DashboardOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_override_view(item));
    }
    views
}

pub fn sum_dashboard_override_view(view: &DashboardOverrideView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_override("code-664", "label-494");
        let view = to_dashboard_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
