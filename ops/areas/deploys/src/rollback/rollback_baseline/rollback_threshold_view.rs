use super::*;

use acme_kit::text;

/// How screens and exports show a rollback threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdView {
    pub code: String,
    pub label_text: String,
    pub pinned: String,
    pub channel: String,
    pub currency: String,
    pub name: String,
    pub approved: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_rollback_threshold_view(item: &RollbackThreshold) -> RollbackThresholdView {
    RollbackThresholdView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        pinned: format!("{}", item.pinned),
        channel: item.channel.clone(),
        currency: item.currency.clone(),
        name: text::truncate(&item.name, 120),
        approved: format!("{}", item.approved),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_threshold_views(items: &[RollbackThreshold]) -> Vec<RollbackThresholdView> {
    let mut views: Vec<RollbackThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_threshold_view(item));
    }
    views
}

pub fn sum_rollback_threshold_view(view: &RollbackThresholdView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_threshold("code-87", "label-98");
        let view = to_rollback_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
