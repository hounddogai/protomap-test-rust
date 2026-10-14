use super::*;

use acme_kit::text;

/// How screens and exports show a release milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneView {
    pub code_text: String,
    pub name_text: String,
    pub quota_units: String,
    pub issuer: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_release_milestone_view(item: &ReleaseMilestone) -> ReleaseMilestoneView {
    ReleaseMilestoneView {
        code_text: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        issuer: item.issuer.clone(),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_milestone_views(items: &[ReleaseMilestone]) -> Vec<ReleaseMilestoneView> {
    let mut views: Vec<ReleaseMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_milestone_view(item));
    }
    views
}

pub fn sum_release_milestone_view(view: &ReleaseMilestoneView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_milestone("code-797", "name-839");
        let view = to_release_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
