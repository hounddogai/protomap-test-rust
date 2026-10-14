use super::*;

use acme_kit::text;

/// How screens and exports show a evidence threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceThresholdView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub cipher_text: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_evidence_threshold_view(item: &EvidenceThreshold) -> EvidenceThresholdView {
    EvidenceThresholdView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        cipher_text: item.cipher.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_threshold_views(items: &[EvidenceThreshold]) -> Vec<EvidenceThresholdView> {
    let mut views: Vec<EvidenceThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_threshold_view(item));
    }
    views
}

pub fn sum_evidence_threshold_view(view: &EvidenceThresholdView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_threshold("code-915", "name-562");
        let view = to_evidence_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
