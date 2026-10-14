use super::*;

use acme_kit::text;

/// How screens and exports show a probe preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreferenceView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub maximum: String,
    pub priority: String,
    pub scope: String,
    pub external_ref: String,
    pub active: String,
    pub key_id: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_probe_preference_view(item: &ProbePreference) -> ProbePreferenceView {
    ProbePreferenceView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        summary: text::truncate(&item.summary, 80),
        maximum: format!("{:.2}", item.maximum),
        priority: item.priority.to_string(),
        scope: item.scope.clone(),
        external_ref: item.external_ref.clone(),
        active: format!("{}", item.active),
        key_id: text::truncate(&item.key_id, 80),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_preference_views(items: &[ProbePreference]) -> Vec<ProbePreferenceView> {
    let mut views: Vec<ProbePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_preference_view(item));
    }
    views
}

pub fn sum_probe_preference_view(view: &ProbePreferenceView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_maximum + (parsed_priority as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_preference("code-409", "name-3");
        let view = to_probe_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
