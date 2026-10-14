use super::*;

use acme_kit::text;

/// How screens and exports show a evidence summary.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSummaryView {
    pub code: String,
    pub name: String,
    pub threshold_text: String,
    pub updated_at_text: String,
    pub port: String,
    pub cluster: String,
    pub duration_minutes: String,
    pub phone: String,
    pub ends_at_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_evidence_summary_view(item: &EvidenceSummary) -> EvidenceSummaryView {
    EvidenceSummaryView {
        code: text::truncate(&item.code, 64),
        name: text::truncate(&item.name, 120),
        threshold_text: format!("{:.2}", item.threshold),
        updated_at_text: item.updated_at.to_string(),
        port: item.port.to_string(),
        cluster: item.cluster.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        phone: item.phone.clone(),
        ends_at_text: item.ends_at.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_summary_views(items: &[EvidenceSummary]) -> Vec<EvidenceSummaryView> {
    let mut views: Vec<EvidenceSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_summary_view(item));
    }
    views
}

pub fn sum_evidence_summary_view(view: &EvidenceSummaryView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_threshold + (parsed_updated_at as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_summary("code-790", "name-698");
        let view = to_evidence_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
