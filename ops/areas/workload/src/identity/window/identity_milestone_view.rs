use super::*;

use acme_kit::text;

/// How screens and exports show a identity milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMilestoneView {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub cluster: String,
    pub channel: String,
    pub external_ref: String,
    pub maximum_text: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_identity_milestone_view(item: &IdentityMilestone) -> IdentityMilestoneView {
    IdentityMilestoneView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        unit: item.unit.clone(),
        cluster: item.cluster.clone(),
        channel: text::truncate(&item.channel, 40),
        external_ref: item.external_ref.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        timezone_text: text::truncate(&item.timezone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_milestone_views(items: &[IdentityMilestone]) -> Vec<IdentityMilestoneView> {
    let mut views: Vec<IdentityMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_milestone_view(item));
    }
    views
}

pub fn sum_identity_milestone_view(view: &IdentityMilestoneView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_milestone("code-650", "label-104");
        let view = to_identity_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
