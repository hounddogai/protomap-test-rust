use super::*;

use acme_kit::text;

/// How screens and exports show a tenant override.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantOverrideView {
    pub code_text: String,
    pub name_text: String,
    pub notes: String,
    pub quantity: String,
    pub channel_text: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_tenant_override_view(item: &TenantOverride) -> TenantOverrideView {
    TenantOverrideView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        notes: text::truncate(&item.notes, 64),
        quantity: item.quantity.to_string(),
        channel_text: item.channel.clone(),
        contact_email_text: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_override_views(items: &[TenantOverride]) -> Vec<TenantOverrideView> {
    let mut views: Vec<TenantOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_override_view(item));
    }
    views
}

pub fn sum_tenant_override_view(view: &TenantOverrideView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_override("code-30", "name-916");
        let view = to_tenant_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
