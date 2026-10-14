use super::*;

/// How screens and exports show a lenient zone preference.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZonePreferenceView {
    pub code: String,
    pub title: String,
    pub floor_value_text: String,
    pub starts_at: String,
    pub source_text: String,
    pub ratio: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_lenient_zone_preference_view(item: &LenientZonePreference) -> LenientZonePreferenceView {
    LenientZonePreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        starts_at: item.starts_at.to_string(),
        source_text: item.source.clone(),
        ratio: format!("{:.2}", item.ratio),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_zone_preference_views(items: &[LenientZonePreference]) -> Vec<LenientZonePreferenceView> {
    let mut views: Vec<LenientZonePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_zone_preference_view(item));
    }
    views
}

pub fn sum_lenient_zone_preference_view(view: &LenientZonePreferenceView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_floor_value + (parsed_starts_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_zone_preference("code-740", "title-378");
        let view = to_lenient_zone_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_zone_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
