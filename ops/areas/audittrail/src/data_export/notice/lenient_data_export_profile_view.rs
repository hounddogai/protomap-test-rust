use super::*;

use acme_kit::text;

/// How screens and exports show a lenient data export profile.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDataExportProfileView {
    pub code: String,
    pub label: String,
    pub revision_text: String,
    pub burst: String,
    pub namespace: String,
    pub segment: String,
    pub margin_text: String,
    pub pinned_text: String,
    pub updated_at: String,
    pub category: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_lenient_data_export_profile_view(item: &LenientDataExportProfile) -> LenientDataExportProfileView {
    LenientDataExportProfileView {
        code: item.code.clone(),
        label: item.label.clone(),
        revision_text: item.revision.to_string(),
        burst: item.burst.to_string(),
        namespace: item.namespace.clone(),
        segment: text::truncate(&item.segment, 120),
        margin_text: format!("{:.2}", item.margin),
        pinned_text: format!("{}", item.pinned),
        updated_at: item.updated_at.to_string(),
        category: text::truncate(&item.category, 64),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_data_export_profile_views(items: &[LenientDataExportProfile]) -> Vec<LenientDataExportProfileView> {
    let mut views: Vec<LenientDataExportProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_data_export_profile_view(item));
    }
    views
}

pub fn sum_lenient_data_export_profile_view(view: &LenientDataExportProfileView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_burst as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_data_export_profile("code-532", "label-948");
        let view = to_lenient_data_export_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_data_export_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
