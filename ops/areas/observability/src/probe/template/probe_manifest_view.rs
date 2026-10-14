use super::*;

use acme_kit::text;

/// How screens and exports show a probe manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeManifestView {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub owner_id_text: String,
    pub revision: String,
    pub starts_at: String,
    pub port_text: String,
    pub category: String,
    pub rotation_days_text: String,
    pub cipher_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_probe_manifest_view(item: &ProbeManifest) -> ProbeManifestView {
    ProbeManifestView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        channel: text::truncate(&item.channel, 120),
        owner_id_text: item.owner_id.clone(),
        revision: item.revision.to_string(),
        starts_at: item.starts_at.to_string(),
        port_text: item.port.to_string(),
        category: item.category.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        cipher_text: item.cipher.clone(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_manifest_views(items: &[ProbeManifest]) -> Vec<ProbeManifestView> {
    let mut views: Vec<ProbeManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_manifest_view(item));
    }
    views
}

pub fn sum_probe_manifest_view(view: &ProbeManifestView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_starts_at as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_manifest("code-792", "label-652");
        let view = to_probe_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
