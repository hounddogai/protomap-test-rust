use super::*;

use acme_kit::text;

/// How screens and exports show a lenient freeze checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeCheckpointView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub port: String,
    pub namespace: String,
    pub minimum_text: String,
    pub amount_text: String,
    pub owner_id: String,
    pub summary_text: String,
    pub category_text: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_lenient_freeze_checkpoint_view(item: &LenientFreezeCheckpoint) -> LenientFreezeCheckpointView {
    LenientFreezeCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        port: item.port.to_string(),
        namespace: item.namespace.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        amount_text: format!("{:.2}", item.amount),
        owner_id: text::truncate(&item.owner_id, 120),
        summary_text: text::truncate(&item.summary, 120),
        category_text: item.category.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_freeze_checkpoint_views(items: &[LenientFreezeCheckpoint]) -> Vec<LenientFreezeCheckpointView> {
    let mut views: Vec<LenientFreezeCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_freeze_checkpoint_view(item));
    }
    views
}

pub fn sum_lenient_freeze_checkpoint_view(view: &LenientFreezeCheckpointView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_port as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_freeze_checkpoint("code-914", "name-783");
        let view = to_lenient_freeze_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_freeze_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
