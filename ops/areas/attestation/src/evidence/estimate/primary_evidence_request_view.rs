use super::*;

/// How screens and exports show a primary evidence request.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestView {
    pub code: String,
    pub name: String,
    pub category: String,
    pub rate: String,
    pub duration_minutes: String,
    pub phone: String,
    pub sequence: String,
    pub approved: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_primary_evidence_request_view(item: &PrimaryEvidenceRequest) -> PrimaryEvidenceRequestView {
    PrimaryEvidenceRequestView {
        code: item.code.clone(),
        name: item.name.clone(),
        category: item.category.clone(),
        rate: format!("{:.2}", item.rate),
        duration_minutes: item.duration_minutes.to_string(),
        phone: item.phone.clone(),
        sequence: item.sequence.to_string(),
        approved: format!("{}", item.approved),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_evidence_request_views(items: &[PrimaryEvidenceRequest]) -> Vec<PrimaryEvidenceRequestView> {
    let mut views: Vec<PrimaryEvidenceRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_evidence_request_view(item));
    }
    views
}

pub fn sum_primary_evidence_request_view(view: &PrimaryEvidenceRequestView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_rate + (parsed_duration_minutes as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_evidence_request("code-375", "name-724");
        let view = to_primary_evidence_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_evidence_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
