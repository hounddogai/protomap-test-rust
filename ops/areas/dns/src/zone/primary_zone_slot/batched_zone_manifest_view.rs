use super::*;

use acme_kit::text;

/// How screens and exports show a batched zone manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneManifestView {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub duration_minutes: String,
    pub starts_at_text: String,
    pub channel: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_batched_zone_manifest_view(item: &BatchedZoneManifest) -> BatchedZoneManifestView {
    BatchedZoneManifestView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        contact_email: item.contact_email.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        starts_at_text: item.starts_at.to_string(),
        channel: item.channel.clone(),
        cipher: text::truncate(&item.cipher, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_zone_manifest_views(items: &[BatchedZoneManifest]) -> Vec<BatchedZoneManifestView> {
    let mut views: Vec<BatchedZoneManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_zone_manifest_view(item));
    }
    views
}

pub fn sum_batched_zone_manifest_view(view: &BatchedZoneManifestView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_zone_manifest("code-339", "label-946");
        let view = to_batched_zone_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_zone_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
