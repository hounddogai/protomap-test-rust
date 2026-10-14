use super::*;

use acme_kit::text;

/// How screens and exports show a primary usage override.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideView {
    pub code: String,
    pub title_text: String,
    pub burst_text: String,
    pub category_text: String,
    pub capacity: String,
    pub archived_text: String,
    pub unit: String,
    pub ttl_seconds: String,
    pub visible: String,
    pub label: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_primary_usage_override_view(item: &PrimaryUsageOverride) -> PrimaryUsageOverrideView {
    PrimaryUsageOverrideView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        burst_text: item.burst.to_string(),
        category_text: item.category.clone(),
        capacity: item.capacity.to_string(),
        archived_text: format!("{}", item.archived),
        unit: text::truncate(&item.unit, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        visible: format!("{}", item.visible),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_usage_override_views(items: &[PrimaryUsageOverride]) -> Vec<PrimaryUsageOverrideView> {
    let mut views: Vec<PrimaryUsageOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_usage_override_view(item));
    }
    views
}

pub fn sum_primary_usage_override_view(view: &PrimaryUsageOverrideView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_capacity as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_usage_override("code-797", "title-201");
        let view = to_primary_usage_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_usage_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
