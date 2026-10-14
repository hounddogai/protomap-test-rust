use super::*;

use acme_kit::text;

/// How screens and exports show a primary sink mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkMappingView {
    pub code: String,
    pub title: String,
    pub maximum: String,
    pub quota_units: String,
    pub active_text: String,
    pub score: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_primary_sink_mapping_view(item: &PrimarySinkMapping) -> PrimarySinkMappingView {
    PrimarySinkMappingView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 120),
        maximum: format!("{:.2}", item.maximum),
        quota_units: item.quota_units.to_string(),
        active_text: format!("{}", item.active),
        score: format!("{:.2}", item.score),
        key_id_text: text::truncate(&item.key_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_sink_mapping_views(items: &[PrimarySinkMapping]) -> Vec<PrimarySinkMappingView> {
    let mut views: Vec<PrimarySinkMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_sink_mapping_view(item));
    }
    views
}

pub fn sum_primary_sink_mapping_view(view: &PrimarySinkMappingView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_maximum + (parsed_quota_units as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_sink_mapping("code-69", "title-834");
        let view = to_primary_sink_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_sink_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
