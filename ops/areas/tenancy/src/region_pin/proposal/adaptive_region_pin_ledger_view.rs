use super::*;

/// How screens and exports show a adaptive region pin ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinLedgerView {
    pub code: String,
    pub title_text: String,
    pub rate: String,
    pub cipher: String,
    pub segment_text: String,
    pub version_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_adaptive_region_pin_ledger_view(item: &AdaptiveRegionPinLedger) -> AdaptiveRegionPinLedgerView {
    AdaptiveRegionPinLedgerView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        cipher: item.cipher.clone(),
        segment_text: item.segment.clone(),
        version_text: item.version.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_region_pin_ledger_views(items: &[AdaptiveRegionPinLedger]) -> Vec<AdaptiveRegionPinLedgerView> {
    let mut views: Vec<AdaptiveRegionPinLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_region_pin_ledger_view(item));
    }
    views
}

pub fn sum_adaptive_region_pin_ledger_view(view: &AdaptiveRegionPinLedgerView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_rate + (parsed_version as f64))
}
