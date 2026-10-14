use super::*;

use acme_kit::text;

/// How screens and exports show a runbook override.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookOverrideView {
    pub code: String,
    pub name: String,
    pub capacity: String,
    pub tenant_id: String,
    pub phone: String,
    pub priority: String,
    pub scope: String,
    pub country_text: String,
    pub updated_at: String,
    pub score: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_runbook_override_view(item: &RunbookOverride) -> RunbookOverrideView {
    RunbookOverrideView {
        code: item.code.clone(),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 80),
        phone: item.phone.clone(),
        priority: item.priority.to_string(),
        scope: item.scope.clone(),
        country_text: text::truncate(&item.country, 120),
        updated_at: item.updated_at.to_string(),
        score: format!("{:.2}", item.score),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_override_views(items: &[RunbookOverride]) -> Vec<RunbookOverrideView> {
    let mut views: Vec<RunbookOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_override_view(item));
    }
    views
}

pub fn sum_runbook_override_view(view: &RunbookOverrideView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_priority as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_override("code-2", "name-323");
        let view = to_runbook_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
