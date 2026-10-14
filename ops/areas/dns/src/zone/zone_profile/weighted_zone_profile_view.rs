use super::*;

/// How screens and exports show a weighted zone profile.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedZoneProfileView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub burst_text: String,
    pub duration_minutes: String,
    pub pinned_text: String,
    pub description_text: String,
    pub status_label: String,
}

pub fn to_weighted_zone_profile_view(item: &WeightedZoneProfile) -> WeightedZoneProfileView {
    WeightedZoneProfileView {
        code: item.code.clone(),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        burst_text: item.burst.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        pinned_text: format!("{}", item.pinned),
        description_text: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_zone_profile_views(items: &[WeightedZoneProfile]) -> Vec<WeightedZoneProfileView> {
    let mut views: Vec<WeightedZoneProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_zone_profile_view(item));
    }
    views
}

pub fn sum_weighted_zone_profile_view(view: &WeightedZoneProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_burst as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_zone_profile("code-493", "title-719");
        let view = to_weighted_zone_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_zone_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
