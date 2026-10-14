use super::*;

use acme_kit::text;

/// How screens and exports show a silence manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceManifestView {
    pub code: String,
    pub label: String,
    pub channel_text: String,
    pub batch_size: String,
    pub margin_text: String,
    pub unit: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_silence_manifest_view(item: &SilenceManifest) -> SilenceManifestView {
    SilenceManifestView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        channel_text: item.channel.clone(),
        batch_size: item.batch_size.to_string(),
        margin_text: format!("{:.2}", item.margin),
        unit: item.unit.clone(),
        summary: text::truncate(&item.summary, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_manifest_views(items: &[SilenceManifest]) -> Vec<SilenceManifestView> {
    let mut views: Vec<SilenceManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_manifest_view(item));
    }
    views
}

pub fn sum_silence_manifest_view(view: &SilenceManifestView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_manifest("code-534", "label-701");
        let view = to_silence_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
