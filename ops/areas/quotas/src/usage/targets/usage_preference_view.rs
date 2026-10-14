use super::*;

use acme_kit::text;

/// How screens and exports show a usage preference.
#[derive(Debug, Clone, PartialEq)]
pub struct UsagePreferenceView {
    pub code: String,
    pub title_text: String,
    pub batch_size: String,
    pub ttl_seconds_text: String,
    pub cluster: String,
    pub visible: String,
    pub minimum: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_usage_preference_view(item: &UsagePreference) -> UsagePreferenceView {
    UsagePreferenceView {
        code: text::truncate(&item.code, 64),
        title_text: text::truncate(&item.title, 64),
        batch_size: item.batch_size.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        cluster: item.cluster.clone(),
        visible: format!("{}", item.visible),
        minimum: format!("{:.2}", item.minimum),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_preference_views(items: &[UsagePreference]) -> Vec<UsagePreferenceView> {
    let mut views: Vec<UsagePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_preference_view(item));
    }
    views
}

pub fn sum_usage_preference_view(view: &UsagePreferenceView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_ttl_seconds as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_preference("code-785", "title-95");
        let view = to_usage_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
