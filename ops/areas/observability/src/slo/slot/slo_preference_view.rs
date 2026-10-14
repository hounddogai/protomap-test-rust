use super::*;

use acme_kit::text;

/// How screens and exports show a slo preference.
#[derive(Debug, Clone, PartialEq)]
pub struct SloPreferenceView {
    pub code_text: String,
    pub label_text: String,
    pub unit: String,
    pub cluster: String,
    pub threshold: String,
    pub grace_days: String,
    pub version: String,
    pub description: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_slo_preference_view(item: &SloPreference) -> SloPreferenceView {
    SloPreferenceView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        unit: item.unit.clone(),
        cluster: item.cluster.clone(),
        threshold: format!("{:.2}", item.threshold),
        grace_days: item.grace_days.to_string(),
        version: item.version.to_string(),
        description: item.description.clone(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_preference_views(items: &[SloPreference]) -> Vec<SloPreferenceView> {
    let mut views: Vec<SloPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_preference_view(item));
    }
    views
}

pub fn sum_slo_preference_view(view: &SloPreferenceView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_threshold + (parsed_grace_days as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_preference("code-873", "label-178");
        let view = to_slo_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
