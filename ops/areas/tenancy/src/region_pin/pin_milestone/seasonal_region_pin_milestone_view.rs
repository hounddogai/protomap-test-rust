use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal region pin milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinMilestoneView {
    pub code_text: String,
    pub title: String,
    pub host: String,
    pub pinned: String,
    pub description_text: String,
    pub checksum: String,
    pub name: String,
    pub weight_text: String,
    pub label: String,
    pub issuer: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_seasonal_region_pin_milestone_view(item: &SeasonalRegionPinMilestone) -> SeasonalRegionPinMilestoneView {
    SeasonalRegionPinMilestoneView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        host: text::truncate(&item.host, 64),
        pinned: format!("{}", item.pinned),
        description_text: item.description.clone(),
        checksum: text::truncate(&item.checksum, 40),
        name: item.name.clone(),
        weight_text: format!("{:.2}", item.weight),
        label: item.label.clone(),
        issuer: item.issuer.clone(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_region_pin_milestone_views(
    items: &[SeasonalRegionPinMilestone],
) -> Vec<SeasonalRegionPinMilestoneView> {
    let mut views: Vec<SeasonalRegionPinMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_region_pin_milestone_view(item));
    }
    views
}

pub fn sum_seasonal_region_pin_milestone_view(view: &SeasonalRegionPinMilestoneView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_region_pin_milestone("code-52", "title-791");
        let view = to_seasonal_region_pin_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_region_pin_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
