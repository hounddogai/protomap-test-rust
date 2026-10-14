use super::*;

use acme_kit::text;

/// How screens and exports show a tenant slot.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSlotView {
    pub code_text: String,
    pub name: String,
    pub expires_at_text: String,
    pub principal: String,
    pub tenant_id: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_tenant_slot_view(item: &TenantSlot) -> TenantSlotView {
    TenantSlotView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        expires_at_text: item.expires_at.to_string(),
        principal: item.principal.clone(),
        tenant_id: item.tenant_id.clone(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_slot_views(items: &[TenantSlot]) -> Vec<TenantSlotView> {
    let mut views: Vec<TenantSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_slot_view(item));
    }
    views
}

pub fn sum_tenant_slot_view(view: &TenantSlotView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_slot("code-435", "name-76");
        let view = to_tenant_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
