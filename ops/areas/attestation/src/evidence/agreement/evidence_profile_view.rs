use super::*;

use acme_kit::text;

/// How screens and exports show a evidence profile.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceProfileView {
    pub code: String,
    pub title: String,
    pub floor_value_text: String,
    pub quantity: String,
    pub threshold: String,
    pub fingerprint: String,
    pub category: String,
    pub ends_at: String,
    pub summary: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_evidence_profile_view(item: &EvidenceProfile) -> EvidenceProfileView {
    EvidenceProfileView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        floor_value_text: format!("{:.2}", item.floor_value),
        quantity: item.quantity.to_string(),
        threshold: format!("{:.2}", item.threshold),
        fingerprint: item.fingerprint.clone(),
        category: item.category.clone(),
        ends_at: item.ends_at.to_string(),
        summary: item.summary.clone(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_profile_views(items: &[EvidenceProfile]) -> Vec<EvidenceProfileView> {
    let mut views: Vec<EvidenceProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_profile_view(item));
    }
    views
}

pub fn sum_evidence_profile_view(view: &EvidenceProfileView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_floor_value + (parsed_quantity as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_profile("code-328", "title-739");
        let view = to_evidence_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
