use super::*;

use acme_kit::text;

/// How screens and exports show a regional egress exception.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionView {
    pub code: String,
    pub name: String,
    pub sequence_text: String,
    pub archived: String,
    pub host_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_regional_egress_exception_view(item: &RegionalEgressException) -> RegionalEgressExceptionView {
    RegionalEgressExceptionView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        sequence_text: item.sequence.to_string(),
        archived: format!("{}", item.archived),
        host_text: item.host.clone(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_egress_exception_views(items: &[RegionalEgressException]) -> Vec<RegionalEgressExceptionView> {
    let mut views: Vec<RegionalEgressExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_egress_exception_view(item));
    }
    views
}

pub fn sum_regional_egress_exception_view(view: &RegionalEgressExceptionView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}
