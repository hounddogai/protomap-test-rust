use super::*;

/// How screens and exports show a data export manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestView {
    pub code: String,
    pub name: String,
    pub expires_at: String,
    pub principal_text: String,
    pub cluster_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_data_export_manifest_view(item: &DataExportManifest) -> DataExportManifestView {
    DataExportManifestView {
        code: item.code.clone(),
        name: item.name.clone(),
        expires_at: item.expires_at.to_string(),
        principal_text: item.principal.clone(),
        cluster_text: item.cluster.clone(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_manifest_views(items: &[DataExportManifest]) -> Vec<DataExportManifestView> {
    let mut views: Vec<DataExportManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_manifest_view(item));
    }
    views
}

pub fn sum_data_export_manifest_view(view: &DataExportManifestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_manifest("code-555", "name-539");
        let view = to_data_export_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
