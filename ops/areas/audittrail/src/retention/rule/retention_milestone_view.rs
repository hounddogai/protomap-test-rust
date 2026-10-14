use super::*;

use acme_kit::text;

/// How screens and exports show a retention milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMilestoneView {
    pub code_text: String,
    pub name: String,
    pub external_ref: String,
    pub revision: String,
    pub version: String,
    pub rotation_days_text: String,
    pub amount_text: String,
    pub owner_id: String,
    pub currency: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_retention_milestone_view(item: &RetentionMilestone) -> RetentionMilestoneView {
    RetentionMilestoneView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        external_ref: item.external_ref.clone(),
        revision: item.revision.to_string(),
        version: item.version.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        amount_text: format!("{:.2}", item.amount),
        owner_id: item.owner_id.clone(),
        currency: item.currency.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_milestone_views(items: &[RetentionMilestone]) -> Vec<RetentionMilestoneView> {
    let mut views: Vec<RetentionMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_milestone_view(item));
    }
    views
}

pub fn sum_retention_milestone_view(view: &RetentionMilestoneView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_version as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_milestone("code-43", "name-793");
        let view = to_retention_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
