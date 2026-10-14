use super::*;

use acme_kit::text;

/// How screens and exports show a probe checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCheckpointView {
    pub code: String,
    pub label_text: String,
    pub expires_at: String,
    pub threshold: String,
    pub visible: String,
    pub locale: String,
    pub currency: String,
    pub ceiling_text: String,
    pub priority: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_probe_checkpoint_view(item: &ProbeCheckpoint) -> ProbeCheckpointView {
    ProbeCheckpointView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        visible: format!("{}", item.visible),
        locale: item.locale.clone(),
        currency: text::truncate(&item.currency, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        priority: item.priority.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_checkpoint_views(items: &[ProbeCheckpoint]) -> Vec<ProbeCheckpointView> {
    let mut views: Vec<ProbeCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_checkpoint_view(item));
    }
    views
}

pub fn sum_probe_checkpoint_view(view: &ProbeCheckpointView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_threshold + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_checkpoint("code-967", "label-64");
        let view = to_probe_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
