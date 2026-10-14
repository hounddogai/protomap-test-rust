use super::*;

use acme_kit::text;

/// How screens and exports show a regional failover checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverCheckpointView {
    pub code: String,
    pub label_text: String,
    pub priority_text: String,
    pub country: String,
    pub checksum: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_regional_failover_checkpoint_view(item: &RegionalFailoverCheckpoint) -> RegionalFailoverCheckpointView {
    RegionalFailoverCheckpointView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        priority_text: item.priority.to_string(),
        country: item.country.clone(),
        checksum: item.checksum.clone(),
        source_text: text::truncate(&item.source, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_failover_checkpoint_views(
    items: &[RegionalFailoverCheckpoint],
) -> Vec<RegionalFailoverCheckpointView> {
    let mut views: Vec<RegionalFailoverCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_failover_checkpoint_view(item));
    }
    views
}

pub fn sum_regional_failover_checkpoint_view(view: &RegionalFailoverCheckpointView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_failover_checkpoint("code-672", "label-199");
        let view = to_regional_failover_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_failover_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
