use super::*;

use acme_kit::text;

/// How screens and exports show a lenient retention checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionCheckpointView {
    pub code: String,
    pub name_text: String,
    pub owner_id: String,
    pub duration_minutes: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_lenient_retention_checkpoint_view(item: &LenientRetentionCheckpoint) -> LenientRetentionCheckpointView {
    LenientRetentionCheckpointView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        owner_id: text::truncate(&item.owner_id, 64),
        duration_minutes: item.duration_minutes.to_string(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_retention_checkpoint_views(
    items: &[LenientRetentionCheckpoint],
) -> Vec<LenientRetentionCheckpointView> {
    let mut views: Vec<LenientRetentionCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_retention_checkpoint_view(item));
    }
    views
}

pub fn sum_lenient_retention_checkpoint_view(view: &LenientRetentionCheckpointView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_retention_checkpoint("code-959", "name-972");
        let view = to_lenient_retention_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_retention_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
