use super::*;

/// How screens and exports show a evidence reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReservationView {
    pub code_text: String,
    pub label_text: String,
    pub visible: String,
    pub updated_at: String,
    pub grace_days: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_evidence_reservation_view(item: &EvidenceReservation) -> EvidenceReservationView {
    EvidenceReservationView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        visible: format!("{}", item.visible),
        updated_at: item.updated_at.to_string(),
        grace_days: item.grace_days.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_reservation_views(items: &[EvidenceReservation]) -> Vec<EvidenceReservationView> {
    let mut views: Vec<EvidenceReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_reservation_view(item));
    }
    views
}

pub fn sum_evidence_reservation_view(view: &EvidenceReservationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_reservation("code-293", "label-131");
        let view = to_evidence_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
