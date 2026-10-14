use super::*;

use acme_kit::text;

/// How screens and exports show a rollback plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlanView {
    pub code: String,
    pub label_text: String,
    pub port: String,
    pub batch_size: String,
    pub version: String,
    pub pinned: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_rollback_plan_view(item: &RollbackPlan) -> RollbackPlanView {
    RollbackPlanView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        port: item.port.to_string(),
        batch_size: item.batch_size.to_string(),
        version: item.version.to_string(),
        pinned: format!("{}", item.pinned),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_plan_views(items: &[RollbackPlan]) -> Vec<RollbackPlanView> {
    let mut views: Vec<RollbackPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_plan_view(item));
    }
    views
}

pub fn sum_rollback_plan_view(view: &RollbackPlanView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_port as f64 + (parsed_batch_size as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_plan("code-858", "label-259");
        let view = to_rollback_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
