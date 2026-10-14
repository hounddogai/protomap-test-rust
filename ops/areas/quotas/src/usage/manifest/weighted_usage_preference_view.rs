use super::*;

use acme_kit::text;

/// How screens and exports show a weighted usage preference.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceView {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub checksum_text: String,
    pub tenant_id: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_weighted_usage_preference_view(item: &WeightedUsagePreference) -> WeightedUsagePreferenceView {
    WeightedUsagePreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        segment: text::truncate(&item.segment, 80),
        checksum_text: item.checksum.clone(),
        tenant_id: item.tenant_id.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_usage_preference_views(items: &[WeightedUsagePreference]) -> Vec<WeightedUsagePreferenceView> {
    let mut views: Vec<WeightedUsagePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_usage_preference_view(item));
    }
    views
}

pub fn sum_weighted_usage_preference_view(view: &WeightedUsagePreferenceView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_usage_preference("code-782", "title-648");
        let view = to_weighted_usage_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_usage_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
