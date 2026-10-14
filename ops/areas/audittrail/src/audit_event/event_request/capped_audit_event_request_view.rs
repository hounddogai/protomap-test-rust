use super::*;

use acme_kit::text;

/// How screens and exports show a capped audit event request.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedAuditEventRequestView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub quota_units: String,
    pub budget: String,
    pub currency_text: String,
    pub discount_text: String,
    pub quantity: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_capped_audit_event_request_view(item: &CappedAuditEventRequest) -> CappedAuditEventRequestView {
    CappedAuditEventRequestView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        quota_units: item.quota_units.to_string(),
        budget: format!("{:.2}", item.budget),
        currency_text: text::truncate(&item.currency, 80),
        discount_text: format!("{:.2}", item.discount),
        quantity: item.quantity.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_audit_event_request_views(items: &[CappedAuditEventRequest]) -> Vec<CappedAuditEventRequestView> {
    let mut views: Vec<CappedAuditEventRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_audit_event_request_view(item));
    }
    views
}

pub fn sum_capped_audit_event_request_view(view: &CappedAuditEventRequestView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_budget + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_audit_event_request("code-693", "name-576");
        let view = to_capped_audit_event_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_audit_event_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
