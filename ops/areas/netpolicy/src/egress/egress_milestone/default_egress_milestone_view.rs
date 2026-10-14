use super::*;

use acme_kit::text;

/// How screens and exports show a default egress milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressMilestoneView {
    pub code: String,
    pub name_text: String,
    pub limit: String,
    pub visible: String,
    pub key_id: String,
    pub revision: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_default_egress_milestone_view(item: &DefaultEgressMilestone) -> DefaultEgressMilestoneView {
    DefaultEgressMilestoneView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        limit: item.limit.to_string(),
        visible: format!("{}", item.visible),
        key_id: item.key_id.clone(),
        revision: item.revision.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_egress_milestone_views(items: &[DefaultEgressMilestone]) -> Vec<DefaultEgressMilestoneView> {
    let mut views: Vec<DefaultEgressMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_egress_milestone_view(item));
    }
    views
}

pub fn sum_default_egress_milestone_view(view: &DefaultEgressMilestoneView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_revision as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_egress_milestone("code-469", "name-501");
        let view = to_default_egress_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_egress_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
