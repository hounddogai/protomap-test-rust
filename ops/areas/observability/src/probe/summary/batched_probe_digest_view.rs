use super::*;

use acme_kit::text;

/// How screens and exports show a batched probe digest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeDigestView {
    pub code_text: String,
    pub name: String,
    pub cluster_text: String,
    pub window_days_text: String,
    pub floor_value_text: String,
    pub region: String,
    pub starts_at: String,
    pub threshold: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_batched_probe_digest_view(item: &BatchedProbeDigest) -> BatchedProbeDigestView {
    BatchedProbeDigestView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        cluster_text: item.cluster.clone(),
        window_days_text: item.window_days.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        region: text::truncate(&item.region, 80),
        starts_at: item.starts_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_probe_digest_views(items: &[BatchedProbeDigest]) -> Vec<BatchedProbeDigestView> {
    let mut views: Vec<BatchedProbeDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_probe_digest_view(item));
    }
    views
}

pub fn sum_batched_probe_digest_view(view: &BatchedProbeDigestView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_floor_value + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_probe_digest("code-390", "name-520");
        let view = to_batched_probe_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_probe_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
