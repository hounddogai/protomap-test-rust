use super::*;

use acme_kit::text;

/// How screens and exports show a default token threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThresholdView {
    pub code: String,
    pub name: String,
    pub duration_minutes: String,
    pub region: String,
    pub updated_at_text: String,
    pub pinned_text: String,
    pub source_text: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_default_token_threshold_view(item: &DefaultTokenThreshold) -> DefaultTokenThresholdView {
    DefaultTokenThresholdView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        duration_minutes: item.duration_minutes.to_string(),
        region: item.region.clone(),
        updated_at_text: item.updated_at.to_string(),
        pinned_text: format!("{}", item.pinned),
        source_text: text::truncate(&item.source, 80),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_token_threshold_views(items: &[DefaultTokenThreshold]) -> Vec<DefaultTokenThresholdView> {
    let mut views: Vec<DefaultTokenThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_token_threshold_view(item));
    }
    views
}

pub fn sum_default_token_threshold_view(view: &DefaultTokenThresholdView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_token_threshold("code-458", "name-215");
        let view = to_default_token_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_token_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
