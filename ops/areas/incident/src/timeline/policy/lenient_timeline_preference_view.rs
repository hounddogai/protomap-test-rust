use super::*;

/// How screens and exports show a lenient timeline preference.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelinePreferenceView {
    pub code_text: String,
    pub label: String,
    pub name: String,
    pub quota_units: String,
    pub ceiling: String,
    pub discount_text: String,
    pub rotation_days_text: String,
    pub expires_at: String,
    pub batch_size_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_lenient_timeline_preference_view(item: &LenientTimelinePreference) -> LenientTimelinePreferenceView {
    LenientTimelinePreferenceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        name: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        discount_text: format!("{:.2}", item.discount),
        rotation_days_text: item.rotation_days.to_string(),
        expires_at: item.expires_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_timeline_preference_views(items: &[LenientTimelinePreference]) -> Vec<LenientTimelinePreferenceView> {
    let mut views: Vec<LenientTimelinePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_timeline_preference_view(item));
    }
    views
}

pub fn sum_lenient_timeline_preference_view(view: &LenientTimelinePreferenceView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ceiling + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_timeline_preference("code-490", "label-926");
        let view = to_lenient_timeline_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_timeline_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
