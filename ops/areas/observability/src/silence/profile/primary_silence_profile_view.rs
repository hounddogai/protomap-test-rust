use super::*;

use acme_kit::text;

/// How screens and exports show a primary silence profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySilenceProfileView {
    pub code: String,
    pub name: String,
    pub created_at_text: String,
    pub expires_at: String,
    pub summary_text: String,
    pub threshold_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_primary_silence_profile_view(item: &PrimarySilenceProfile) -> PrimarySilenceProfileView {
    PrimarySilenceProfileView {
        code: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        created_at_text: item.created_at.to_string(),
        expires_at: item.expires_at.to_string(),
        summary_text: item.summary.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_silence_profile_views(items: &[PrimarySilenceProfile]) -> Vec<PrimarySilenceProfileView> {
    let mut views: Vec<PrimarySilenceProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_silence_profile_view(item));
    }
    views
}

pub fn sum_primary_silence_profile_view(view: &PrimarySilenceProfileView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_expires_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_silence_profile("code-775", "name-615");
        let view = to_primary_silence_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_silence_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
