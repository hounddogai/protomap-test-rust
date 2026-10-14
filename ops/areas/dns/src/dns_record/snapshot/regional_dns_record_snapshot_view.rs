use super::*;

use acme_kit::text;

/// How screens and exports show a regional dns record snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSnapshotView {
    pub code: String,
    pub name: String,
    pub visible: String,
    pub grace_days: String,
    pub tenant_id: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_regional_dns_record_snapshot_view(item: &RegionalDnsRecordSnapshot) -> RegionalDnsRecordSnapshotView {
    RegionalDnsRecordSnapshotView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        grace_days: item.grace_days.to_string(),
        tenant_id: item.tenant_id.clone(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_dns_record_snapshot_views(
    items: &[RegionalDnsRecordSnapshot],
) -> Vec<RegionalDnsRecordSnapshotView> {
    let mut views: Vec<RegionalDnsRecordSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_dns_record_snapshot_view(item));
    }
    views
}

pub fn sum_regional_dns_record_snapshot_view(view: &RegionalDnsRecordSnapshotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}
