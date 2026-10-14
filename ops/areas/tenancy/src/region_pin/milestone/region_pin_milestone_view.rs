use super::*;

use acme_kit::text;

/// How screens and exports show a region pin milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestoneView {
    pub code_text: String,
    pub label: String,
    pub floor_value: String,
    pub ratio_text: String,
    pub name: String,
    pub rotation_days_text: String,
    pub limit: String,
    pub currency: String,
    pub updated_at: String,
    pub created_at: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_region_pin_milestone_view(item: &RegionPinMilestone) -> RegionPinMilestoneView {
    RegionPinMilestoneView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        ratio_text: format!("{:.2}", item.ratio),
        name: item.name.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        limit: item.limit.to_string(),
        currency: text::truncate(&item.currency, 64),
        updated_at: item.updated_at.to_string(),
        created_at: item.created_at.to_string(),
        title: text::truncate(&item.title, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_milestone_views(items: &[RegionPinMilestone]) -> Vec<RegionPinMilestoneView> {
    let mut views: Vec<RegionPinMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_milestone_view(item));
    }
    views
}

pub fn sum_region_pin_milestone_view(view: &RegionPinMilestoneView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_floor_value + parsed_ratio + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_milestone("code-816", "label-29");
        let view = to_region_pin_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
