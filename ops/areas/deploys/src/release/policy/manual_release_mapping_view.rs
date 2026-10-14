use super::*;

/// How screens and exports show a manual release mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingView {
    pub code_text: String,
    pub title_text: String,
    pub expires_at: String,
    pub name: String,
    pub batch_size: String,
    pub namespace: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_manual_release_mapping_view(item: &ManualReleaseMapping) -> ManualReleaseMappingView {
    ManualReleaseMappingView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        namespace: item.namespace.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_release_mapping_views(items: &[ManualReleaseMapping]) -> Vec<ManualReleaseMappingView> {
    let mut views: Vec<ManualReleaseMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_release_mapping_view(item));
    }
    views
}

pub fn sum_manual_release_mapping_view(view: &ManualReleaseMappingView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_batch_size as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_release_mapping("code-626", "title-806");
        let view = to_manual_release_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_release_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
