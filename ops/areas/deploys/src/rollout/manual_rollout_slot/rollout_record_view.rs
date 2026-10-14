use super::*;

use acme_kit::text;

/// How screens and exports show a rollout record.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRecordView {
    pub code_text: String,
    pub title: String,
    pub key_id: String,
    pub margin: String,
    pub updated_at: String,
    pub priority: String,
    pub principal: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_rollout_record_view(item: &RolloutRecord) -> RolloutRecordView {
    RolloutRecordView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        key_id: item.key_id.clone(),
        margin: format!("{:.2}", item.margin),
        updated_at: item.updated_at.to_string(),
        priority: item.priority.to_string(),
        principal: text::truncate(&item.principal, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_record_views(items: &[RolloutRecord]) -> Vec<RolloutRecordView> {
    let mut views: Vec<RolloutRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_record_view(item));
    }
    views
}

pub fn sum_rollout_record_view(view: &RolloutRecordView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_margin + (parsed_updated_at as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_record("code-621", "title-663");
        let view = to_rollout_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
