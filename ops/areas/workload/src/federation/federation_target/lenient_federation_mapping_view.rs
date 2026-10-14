use super::*;

use acme_kit::text;

/// How screens and exports show a lenient federation mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFederationMappingView {
    pub code_text: String,
    pub title: String,
    pub updated_at: String,
    pub locale: String,
    pub approved: String,
    pub threshold_text: String,
    pub rotation_days: String,
    pub maximum_text: String,
    pub attempts: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_lenient_federation_mapping_view(item: &LenientFederationMapping) -> LenientFederationMappingView {
    LenientFederationMappingView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 64),
        updated_at: item.updated_at.to_string(),
        locale: item.locale.clone(),
        approved: format!("{}", item.approved),
        threshold_text: format!("{:.2}", item.threshold),
        rotation_days: item.rotation_days.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        attempts: item.attempts.to_string(),
        category: text::truncate(&item.category, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_federation_mapping_views(items: &[LenientFederationMapping]) -> Vec<LenientFederationMappingView> {
    let mut views: Vec<LenientFederationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_federation_mapping_view(item));
    }
    views
}

pub fn sum_lenient_federation_mapping_view(view: &LenientFederationMappingView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_threshold + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_federation_mapping("code-516", "title-176");
        let view = to_lenient_federation_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_federation_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
