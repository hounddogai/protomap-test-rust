use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled region pin profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRegionPinProfileView {
    pub code: String,
    pub name: String,
    pub approved: String,
    pub issuer_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_scheduled_region_pin_profile_view(item: &ScheduledRegionPinProfile) -> ScheduledRegionPinProfileView {
    ScheduledRegionPinProfileView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        approved: format!("{}", item.approved),
        issuer_text: item.issuer.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_region_pin_profile_views(
    items: &[ScheduledRegionPinProfile],
) -> Vec<ScheduledRegionPinProfileView> {
    let mut views: Vec<ScheduledRegionPinProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_region_pin_profile_view(item));
    }
    views
}

pub fn sum_scheduled_region_pin_profile_view(view: &ScheduledRegionPinProfileView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_region_pin_profile("code-943", "name-748");
        let view = to_scheduled_region_pin_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_region_pin_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
