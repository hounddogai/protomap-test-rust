use super::*;

use acme_kit::text;

/// How screens and exports show a rolling probe checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub pinned_text: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_rolling_probe_checkpoint_view(item: &RollingProbeCheckpoint) -> RollingProbeCheckpointView {
    RollingProbeCheckpointView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        ceiling: format!("{:.2}", item.ceiling),
        pinned_text: format!("{}", item.pinned),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_probe_checkpoint_views(items: &[RollingProbeCheckpoint]) -> Vec<RollingProbeCheckpointView> {
    let mut views: Vec<RollingProbeCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_probe_checkpoint_view(item));
    }
    views
}

pub fn sum_rolling_probe_checkpoint_view(view: &RollingProbeCheckpointView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_ceiling + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_probe_checkpoint("code-230", "name-965");
        let view = to_rolling_probe_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_probe_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
