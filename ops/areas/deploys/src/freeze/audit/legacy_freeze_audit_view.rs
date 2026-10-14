use super::*;

use acme_kit::text;

/// How screens and exports show a legacy freeze audit.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeAuditView {
    pub code: String,
    pub title: String,
    pub starts_at_text: String,
    pub floor_value: String,
    pub quantity_text: String,
    pub pinned: String,
    pub phone: String,
    pub issuer_text: String,
    pub notes: String,
    pub description: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_legacy_freeze_audit_view(item: &LegacyFreezeAudit) -> LegacyFreezeAuditView {
    LegacyFreezeAuditView {
        code: item.code.clone(),
        title: item.title.clone(),
        starts_at_text: item.starts_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        quantity_text: item.quantity.to_string(),
        pinned: format!("{}", item.pinned),
        phone: item.phone.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        notes: text::truncate(&item.notes, 120),
        description: text::truncate(&item.description, 64),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_freeze_audit_views(items: &[LegacyFreezeAudit]) -> Vec<LegacyFreezeAuditView> {
    let mut views: Vec<LegacyFreezeAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_freeze_audit_view(item));
    }
    views
}

pub fn sum_legacy_freeze_audit_view(view: &LegacyFreezeAuditView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_floor_value + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_freeze_audit("code-876", "title-87");
        let view = to_legacy_freeze_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_freeze_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
