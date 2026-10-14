use super::*;

use acme_kit::text;

/// How screens and exports show a regional delegation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneView {
    pub code: String,
    pub name: String,
    pub margin_text: String,
    pub phone: String,
    pub tenant_id: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_regional_delegation_milestone_view(item: &RegionalDelegationMilestone) -> RegionalDelegationMilestoneView {
    RegionalDelegationMilestoneView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        phone: item.phone.clone(),
        tenant_id: item.tenant_id.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_delegation_milestone_views(
    items: &[RegionalDelegationMilestone],
) -> Vec<RegionalDelegationMilestoneView> {
    let mut views: Vec<RegionalDelegationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_delegation_milestone_view(item));
    }
    views
}

pub fn sum_regional_delegation_milestone_view(view: &RegionalDelegationMilestoneView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_margin + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_delegation_milestone("code-634", "name-606");
        let view = to_regional_delegation_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_delegation_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
