use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled freeze group.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeGroupView {
    pub code: String,
    pub label: String,
    pub score_text: String,
    pub retries: String,
    pub tenant_id: String,
    pub version: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_scheduled_freeze_group_view(item: &ScheduledFreezeGroup) -> ScheduledFreezeGroupView {
    ScheduledFreezeGroupView {
        code: item.code.clone(),
        label: item.label.clone(),
        score_text: format!("{:.2}", item.score),
        retries: item.retries.to_string(),
        tenant_id: item.tenant_id.clone(),
        version: item.version.to_string(),
        description: text::truncate(&item.description, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_freeze_group_views(items: &[ScheduledFreezeGroup]) -> Vec<ScheduledFreezeGroupView> {
    let mut views: Vec<ScheduledFreezeGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_freeze_group_view(item));
    }
    views
}

pub fn sum_scheduled_freeze_group_view(view: &ScheduledFreezeGroupView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_score + (parsed_retries as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_freeze_group("code-770", "label-303");
        let view = to_scheduled_freeze_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_freeze_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
