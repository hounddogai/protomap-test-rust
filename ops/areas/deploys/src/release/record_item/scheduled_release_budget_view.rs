use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled release budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseBudgetView {
    pub code: String,
    pub title: String,
    pub capacity: String,
    pub external_ref: String,
    pub ttl_seconds: String,
    pub margin: String,
    pub active_text: String,
    pub checksum: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_scheduled_release_budget_view(item: &ScheduledReleaseBudget) -> ScheduledReleaseBudgetView {
    ScheduledReleaseBudgetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        capacity: item.capacity.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        margin: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        checksum: item.checksum.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_release_budget_views(items: &[ScheduledReleaseBudget]) -> Vec<ScheduledReleaseBudgetView> {
    let mut views: Vec<ScheduledReleaseBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_release_budget_view(item));
    }
    views
}

pub fn sum_scheduled_release_budget_view(view: &ScheduledReleaseBudgetView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_ttl_seconds as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_release_budget("code-120", "title-953");
        let view = to_scheduled_release_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_release_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
