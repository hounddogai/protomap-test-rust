use super::*;

use acme_kit::text;

/// How screens and exports show a rollback checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointView {
    pub code: String,
    pub name: String,
    pub quota_units: String,
    pub margin: String,
    pub starts_at_text: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_rollback_checkpoint_view(item: &RollbackCheckpoint) -> RollbackCheckpointView {
    RollbackCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        margin: format!("{:.2}", item.margin),
        starts_at_text: item.starts_at.to_string(),
        currency_text: text::truncate(&item.currency, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_checkpoint_views(items: &[RollbackCheckpoint]) -> Vec<RollbackCheckpointView> {
    let mut views: Vec<RollbackCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_checkpoint_view(item));
    }
    views
}

pub fn sum_rollback_checkpoint_view(view: &RollbackCheckpointView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_margin + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_checkpoint("code-796", "name-528");
        let view = to_rollback_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
