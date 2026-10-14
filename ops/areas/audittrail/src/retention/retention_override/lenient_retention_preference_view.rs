use super::*;

use acme_kit::text;

/// How screens and exports show a lenient retention preference.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionPreferenceView {
    pub code: String,
    pub label: String,
    pub floor_value: String,
    pub batch_size: String,
    pub key_id_text: String,
    pub cipher: String,
    pub description: String,
    pub burst_text: String,
    pub score: String,
    pub ends_at: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_lenient_retention_preference_view(item: &LenientRetentionPreference) -> LenientRetentionPreferenceView {
    LenientRetentionPreferenceView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        batch_size: item.batch_size.to_string(),
        key_id_text: item.key_id.clone(),
        cipher: text::truncate(&item.cipher, 64),
        description: text::truncate(&item.description, 64),
        burst_text: item.burst.to_string(),
        score: format!("{:.2}", item.score),
        ends_at: item.ends_at.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_retention_preference_views(
    items: &[LenientRetentionPreference],
) -> Vec<LenientRetentionPreferenceView> {
    let mut views: Vec<LenientRetentionPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_retention_preference_view(item));
    }
    views
}

pub fn sum_lenient_retention_preference_view(view: &LenientRetentionPreferenceView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_floor_value + (parsed_batch_size as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_retention_preference("code-421", "label-78");
        let view = to_lenient_retention_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_retention_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
