use super::*;

use acme_kit::text;

/// How screens and exports show a regional approval group.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalGroupView {
    pub code: String,
    pub title: String,
    pub starts_at: String,
    pub fingerprint: String,
    pub budget: String,
    pub duration_minutes: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_regional_approval_group_view(item: &RegionalApprovalGroup) -> RegionalApprovalGroupView {
    RegionalApprovalGroupView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 40),
        starts_at: item.starts_at.to_string(),
        fingerprint: item.fingerprint.clone(),
        budget: format!("{:.2}", item.budget),
        duration_minutes: item.duration_minutes.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_approval_group_views(items: &[RegionalApprovalGroup]) -> Vec<RegionalApprovalGroupView> {
    let mut views: Vec<RegionalApprovalGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_approval_group_view(item));
    }
    views
}

pub fn sum_regional_approval_group_view(view: &RegionalApprovalGroupView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_budget + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_approval_group("code-79", "title-481");
        let view = to_regional_approval_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_approval_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
