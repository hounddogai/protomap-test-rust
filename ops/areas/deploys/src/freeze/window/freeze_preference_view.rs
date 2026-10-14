use super::*;

use acme_kit::text;

/// How screens and exports show a freeze preference.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezePreferenceView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub title: String,
    pub discount_text: String,
    pub capacity: String,
    pub starts_at: String,
    pub channel_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_freeze_preference_view(item: &FreezePreference) -> FreezePreferenceView {
    FreezePreferenceView {
        code: item.code.clone(),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        capacity: item.capacity.to_string(),
        starts_at: item.starts_at.to_string(),
        channel_text: text::truncate(&item.channel, 80),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_preference_views(items: &[FreezePreference]) -> Vec<FreezePreferenceView> {
    let mut views: Vec<FreezePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_preference_view(item));
    }
    views
}

pub fn sum_freeze_preference_view(view: &FreezePreferenceView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_discount + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_preference("code-205", "label-75");
        let view = to_freeze_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
