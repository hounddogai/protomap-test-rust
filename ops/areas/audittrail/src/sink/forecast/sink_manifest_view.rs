use super::*;

/// How screens and exports show a sink manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkManifestView {
    pub code: String,
    pub label_text: String,
    pub discount: String,
    pub ratio: String,
    pub expires_at: String,
    pub batch_size: String,
    pub currency: String,
    pub quota_units: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_sink_manifest_view(item: &SinkManifest) -> SinkManifestView {
    SinkManifestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        ratio: format!("{:.2}", item.ratio),
        expires_at: item.expires_at.to_string(),
        batch_size: item.batch_size.to_string(),
        currency: item.currency.clone(),
        quota_units: item.quota_units.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_manifest_views(items: &[SinkManifest]) -> Vec<SinkManifestView> {
    let mut views: Vec<SinkManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_manifest_view(item));
    }
    views
}

pub fn sum_sink_manifest_view(view: &SinkManifestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_discount + parsed_ratio + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_manifest("code-191", "label-453");
        let view = to_sink_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
