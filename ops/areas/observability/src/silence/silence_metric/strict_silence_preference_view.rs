use super::*;

/// How screens and exports show a strict silence preference.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreferenceView {
    pub code_text: String,
    pub name: String,
    pub version: String,
    pub revision_text: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_strict_silence_preference_view(item: &StrictSilencePreference) -> StrictSilencePreferenceView {
    StrictSilencePreferenceView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        version: item.version.to_string(),
        revision_text: item.revision.to_string(),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_silence_preference_views(items: &[StrictSilencePreference]) -> Vec<StrictSilencePreferenceView> {
    let mut views: Vec<StrictSilencePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_silence_preference_view(item));
    }
    views
}

pub fn sum_strict_silence_preference_view(view: &StrictSilencePreferenceView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_version as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_silence_preference("code-898", "name-363");
        let view = to_strict_silence_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_silence_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
