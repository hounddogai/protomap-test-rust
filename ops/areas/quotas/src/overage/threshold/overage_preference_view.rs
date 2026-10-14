use super::*;

use acme_kit::text;

/// How screens and exports show a overage preference.
#[derive(Debug, Clone, PartialEq)]
pub struct OveragePreferenceView {
    pub code: String,
    pub title: String,
    pub rotation_days: String,
    pub locale: String,
    pub floor_value_text: String,
    pub duration_minutes: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_overage_preference_view(item: &OveragePreference) -> OveragePreferenceView {
    OveragePreferenceView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        rotation_days: item.rotation_days.to_string(),
        locale: text::truncate(&item.locale, 64),
        floor_value_text: format!("{:.2}", item.floor_value),
        duration_minutes: item.duration_minutes.to_string(),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_preference_views(items: &[OveragePreference]) -> Vec<OveragePreferenceView> {
    let mut views: Vec<OveragePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_preference_view(item));
    }
    views
}

pub fn sum_overage_preference_view(view: &OveragePreferenceView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_floor_value + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_preference("code-61", "title-338");
        let view = to_overage_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
