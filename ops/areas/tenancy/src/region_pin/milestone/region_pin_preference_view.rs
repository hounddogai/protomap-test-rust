use super::*;

use acme_kit::text;

/// How screens and exports show a region pin preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPreferenceView {
    pub code_text: String,
    pub title_text: String,
    pub owner_id: String,
    pub principal: String,
    pub name: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_region_pin_preference_view(item: &RegionPinPreference) -> RegionPinPreferenceView {
    RegionPinPreferenceView {
        code_text: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 120),
        owner_id: item.owner_id.clone(),
        principal: text::truncate(&item.principal, 120),
        name: text::truncate(&item.name, 80),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_preference_views(items: &[RegionPinPreference]) -> Vec<RegionPinPreferenceView> {
    let mut views: Vec<RegionPinPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_preference_view(item));
    }
    views
}

pub fn sum_region_pin_preference_view(view: &RegionPinPreferenceView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_preference("code-263", "title-158");
        let view = to_region_pin_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
