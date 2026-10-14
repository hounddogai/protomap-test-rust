use super::*;

use acme_kit::text;

/// How screens and exports show a regional silence variance.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceVarianceView {
    pub code_text: String,
    pub label: String,
    pub ttl_seconds: String,
    pub margin: String,
    pub namespace: String,
    pub created_at: String,
    pub quantity: String,
    pub reviewer_text: String,
    pub ends_at_text: String,
    pub floor_value: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_regional_silence_variance_view(item: &RegionalSilenceVariance) -> RegionalSilenceVarianceView {
    RegionalSilenceVarianceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        margin: format!("{:.2}", item.margin),
        namespace: text::truncate(&item.namespace, 64),
        created_at: item.created_at.to_string(),
        quantity: item.quantity.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 120),
        ends_at_text: item.ends_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_silence_variance_views(items: &[RegionalSilenceVariance]) -> Vec<RegionalSilenceVarianceView> {
    let mut views: Vec<RegionalSilenceVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_silence_variance_view(item));
    }
    views
}

pub fn sum_regional_silence_variance_view(view: &RegionalSilenceVarianceView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_margin + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_silence_variance("code-724", "label-363");
        let view = to_regional_silence_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_silence_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
