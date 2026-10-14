use super::*;

use acme_kit::text;

/// How screens and exports show a default silence preference.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreferenceView {
    pub code: String,
    pub label: String,
    pub revision: String,
    pub rotation_days_text: String,
    pub ratio: String,
    pub sequence_text: String,
    pub ttl_seconds: String,
    pub description_text: String,
    pub approved: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_default_silence_preference_view(item: &DefaultSilencePreference) -> DefaultSilencePreferenceView {
    DefaultSilencePreferenceView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        revision: item.revision.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        sequence_text: item.sequence.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        description_text: text::truncate(&item.description, 80),
        approved: format!("{}", item.approved),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_silence_preference_views(items: &[DefaultSilencePreference]) -> Vec<DefaultSilencePreferenceView> {
    let mut views: Vec<DefaultSilencePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_silence_preference_view(item));
    }
    views
}

pub fn sum_default_silence_preference_view(view: &DefaultSilencePreferenceView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_rotation_days as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_silence_preference("code-495", "label-969");
        let view = to_default_silence_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_silence_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
