use super::*;

use acme_kit::text;

/// How screens and exports show a primary verifier mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierMappingView {
    pub code: String,
    pub title: String,
    pub quota_units: String,
    pub audience: String,
    pub weight: String,
    pub discount_text: String,
    pub summary: String,
    pub phone: String,
    pub label: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_primary_verifier_mapping_view(item: &PrimaryVerifierMapping) -> PrimaryVerifierMappingView {
    PrimaryVerifierMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        quota_units: item.quota_units.to_string(),
        audience: item.audience.clone(),
        weight: format!("{:.2}", item.weight),
        discount_text: format!("{:.2}", item.discount),
        summary: text::truncate(&item.summary, 120),
        phone: item.phone.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_verifier_mapping_views(items: &[PrimaryVerifierMapping]) -> Vec<PrimaryVerifierMappingView> {
    let mut views: Vec<PrimaryVerifierMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_verifier_mapping_view(item));
    }
    views
}

pub fn sum_primary_verifier_mapping_view(view: &PrimaryVerifierMappingView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_weight + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_verifier_mapping("code-256", "title-383");
        let view = to_primary_verifier_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_verifier_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
