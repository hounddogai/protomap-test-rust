use super::*;

use acme_kit::text;

/// How screens and exports show a peering exception.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionView {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub summary: String,
    pub port: String,
    pub revision: String,
    pub notes: String,
    pub updated_at_text: String,
    pub source_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_peering_exception_view(item: &PeeringException) -> PeeringExceptionView {
    PeeringExceptionView {
        code: item.code.clone(),
        label: item.label.clone(),
        reviewer: item.reviewer.clone(),
        summary: text::truncate(&item.summary, 80),
        port: item.port.to_string(),
        revision: item.revision.to_string(),
        notes: item.notes.clone(),
        updated_at_text: item.updated_at.to_string(),
        source_text: text::truncate(&item.source, 64),
        category: text::truncate(&item.category, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_exception_views(items: &[PeeringException]) -> Vec<PeeringExceptionView> {
    let mut views: Vec<PeeringExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_exception_view(item));
    }
    views
}

pub fn sum_peering_exception_view(view: &PeeringExceptionView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_revision as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_exception("code-871", "label-218");
        let view = to_peering_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
