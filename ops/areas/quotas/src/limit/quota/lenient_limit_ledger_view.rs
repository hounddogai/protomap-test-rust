use super::*;

/// How screens and exports show a lenient limit ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitLedgerView {
    pub code_text: String,
    pub name: String,
    pub label_text: String,
    pub duration_minutes: String,
    pub owner_id: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_lenient_limit_ledger_view(item: &LenientLimitLedger) -> LenientLimitLedgerView {
    LenientLimitLedgerView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        label_text: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        owner_id: item.owner_id.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_limit_ledger_views(items: &[LenientLimitLedger]) -> Vec<LenientLimitLedgerView> {
    let mut views: Vec<LenientLimitLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_limit_ledger_view(item));
    }
    views
}

pub fn sum_lenient_limit_ledger_view(view: &LenientLimitLedgerView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_ratio)
}
