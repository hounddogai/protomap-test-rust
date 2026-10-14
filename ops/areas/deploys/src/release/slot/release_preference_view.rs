use super::*;

use acme_kit::text;

/// How screens and exports show a release preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePreferenceView {
    pub code: String,
    pub label: String,
    pub phone_text: String,
    pub owner_id: String,
    pub principal: String,
    pub region_text: String,
    pub active_text: String,
    pub version: String,
    pub approved_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_release_preference_view(item: &ReleasePreference) -> ReleasePreferenceView {
    ReleasePreferenceView {
        code: item.code.clone(),
        label: item.label.clone(),
        phone_text: item.phone.clone(),
        owner_id: item.owner_id.clone(),
        principal: item.principal.clone(),
        region_text: text::truncate(&item.region, 120),
        active_text: format!("{}", item.active),
        version: item.version.to_string(),
        approved_text: format!("{}", item.approved),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_preference_views(items: &[ReleasePreference]) -> Vec<ReleasePreferenceView> {
    let mut views: Vec<ReleasePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_preference_view(item));
    }
    views
}

pub fn sum_release_preference_view(view: &ReleasePreferenceView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_version as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_preference("code-916", "label-168");
        let view = to_release_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
