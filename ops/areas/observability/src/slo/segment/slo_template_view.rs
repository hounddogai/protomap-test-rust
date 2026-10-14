use super::*;

/// How screens and exports show a slo template.
#[derive(Debug, Clone, PartialEq)]
pub struct SloTemplateView {
    pub code_text: String,
    pub name: String,
    pub expires_at_text: String,
    pub key_id: String,
    pub quantity_text: String,
    pub scope: String,
    pub contact_email_text: String,
    pub country: String,
    pub amount: String,
    pub ceiling: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_slo_template_view(item: &SloTemplate) -> SloTemplateView {
    SloTemplateView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        expires_at_text: item.expires_at.to_string(),
        key_id: item.key_id.clone(),
        quantity_text: item.quantity.to_string(),
        scope: item.scope.clone(),
        contact_email_text: item.contact_email.clone(),
        country: item.country.clone(),
        amount: format!("{:.2}", item.amount),
        ceiling: format!("{:.2}", item.ceiling),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_template_views(items: &[SloTemplate]) -> Vec<SloTemplateView> {
    let mut views: Vec<SloTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_template_view(item));
    }
    views
}

pub fn sum_slo_template_view(view: &SloTemplateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_quantity as f64) + parsed_amount)
}
