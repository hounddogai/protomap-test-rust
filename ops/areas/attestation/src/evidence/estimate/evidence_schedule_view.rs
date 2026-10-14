use super::*;

use acme_kit::text;

/// How screens and exports show a evidence schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleView {
    pub code_text: String,
    pub label_text: String,
    pub quantity: String,
    pub title: String,
    pub retries: String,
    pub cluster_text: String,
    pub country: String,
    pub approved_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_evidence_schedule_view(item: &EvidenceSchedule) -> EvidenceScheduleView {
    EvidenceScheduleView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        quantity: item.quantity.to_string(),
        title: item.title.clone(),
        retries: item.retries.to_string(),
        cluster_text: text::truncate(&item.cluster, 40),
        country: text::truncate(&item.country, 80),
        approved_text: format!("{}", item.approved),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_schedule_views(items: &[EvidenceSchedule]) -> Vec<EvidenceScheduleView> {
    let mut views: Vec<EvidenceScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_schedule_view(item));
    }
    views
}

pub fn sum_evidence_schedule_view(view: &EvidenceScheduleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_retries as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_schedule("code-699", "label-602");
        let view = to_evidence_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
