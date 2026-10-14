use super::*;

use acme_kit::text;

/// How screens and exports show a zone milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMilestoneView {
    pub code_text: String,
    pub title: String,
    pub active: String,
    pub notes: String,
    pub created_at_text: String,
    pub quota_units_text: String,
    pub floor_value: String,
    pub audience_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_zone_milestone_view(item: &ZoneMilestone) -> ZoneMilestoneView {
    ZoneMilestoneView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        active: format!("{}", item.active),
        notes: item.notes.clone(),
        created_at_text: item.created_at.to_string(),
        quota_units_text: item.quota_units.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        audience_text: text::truncate(&item.audience, 64),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_milestone_views(items: &[ZoneMilestone]) -> Vec<ZoneMilestoneView> {
    let mut views: Vec<ZoneMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_milestone_view(item));
    }
    views
}

pub fn sum_zone_milestone_view(view: &ZoneMilestoneView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quota_units as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_milestone("code-393", "title-37");
        let view = to_zone_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
