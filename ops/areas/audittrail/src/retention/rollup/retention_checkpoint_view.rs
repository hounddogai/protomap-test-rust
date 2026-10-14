use super::*;

use acme_kit::text;

/// How screens and exports show a retention checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpointView {
    pub code: String,
    pub title_text: String,
    pub created_at_text: String,
    pub threshold: String,
    pub checksum_text: String,
    pub weight: String,
    pub capacity: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_retention_checkpoint_view(item: &RetentionCheckpoint) -> RetentionCheckpointView {
    RetentionCheckpointView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        created_at_text: item.created_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        checksum_text: text::truncate(&item.checksum, 40),
        weight: format!("{:.2}", item.weight),
        capacity: item.capacity.to_string(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_checkpoint_views(items: &[RetentionCheckpoint]) -> Vec<RetentionCheckpointView> {
    let mut views: Vec<RetentionCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_checkpoint_view(item));
    }
    views
}

pub fn sum_retention_checkpoint_view(view: &RetentionCheckpointView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_threshold + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_checkpoint("code-811", "title-664");
        let view = to_retention_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
