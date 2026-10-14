use super::*;

/// How screens and exports show a region pin profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProfileView {
    pub code: String,
    pub name_text: String,
    pub pinned: String,
    pub ends_at_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_region_pin_profile_view(item: &RegionPinProfile) -> RegionPinProfileView {
    RegionPinProfileView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        pinned: format!("{}", item.pinned),
        ends_at_text: item.ends_at.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_profile_views(items: &[RegionPinProfile]) -> Vec<RegionPinProfileView> {
    let mut views: Vec<RegionPinProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_profile_view(item));
    }
    views
}

pub fn sum_region_pin_profile_view(view: &RegionPinProfileView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_profile("code-363", "name-855");
        let view = to_region_pin_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
