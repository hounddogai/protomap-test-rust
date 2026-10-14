use super::*;

use acme_kit::text;

/// How screens and exports show a approval checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointView {
    pub code: String,
    pub title: String,
    pub limit_text: String,
    pub timezone: String,
    pub locale: String,
    pub audience: String,
    pub rotation_days: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_approval_checkpoint_view(item: &ApprovalCheckpoint) -> ApprovalCheckpointView {
    ApprovalCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        limit_text: item.limit.to_string(),
        timezone: text::truncate(&item.timezone, 120),
        locale: text::truncate(&item.locale, 64),
        audience: item.audience.clone(),
        rotation_days: item.rotation_days.to_string(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_checkpoint_views(items: &[ApprovalCheckpoint]) -> Vec<ApprovalCheckpointView> {
    let mut views: Vec<ApprovalCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_checkpoint_view(item));
    }
    views
}

pub fn sum_approval_checkpoint_view(view: &ApprovalCheckpointView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_rotation_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_checkpoint("code-676", "title-902");
        let view = to_approval_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
