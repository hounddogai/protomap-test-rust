use super::*;

use acme_kit::text;

/// How screens and exports show a evidence exception.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceExceptionView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub retries_text: String,
    pub capacity: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_evidence_exception_view(item: &EvidenceException) -> EvidenceExceptionView {
    EvidenceExceptionView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        port: item.port.to_string(),
        retries_text: item.retries.to_string(),
        capacity: item.capacity.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_exception_views(items: &[EvidenceException]) -> Vec<EvidenceExceptionView> {
    let mut views: Vec<EvidenceExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_exception_view(item));
    }
    views
}

pub fn sum_evidence_exception_view(view: &EvidenceExceptionView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_port as f64 + (parsed_retries as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_exception("code-20", "label-480");
        let view = to_evidence_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
