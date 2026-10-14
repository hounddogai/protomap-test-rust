use super::*;

use acme_kit::text;

/// How screens and exports show a query template.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryTemplateView {
    pub code: String,
    pub title: String,
    pub phone_text: String,
    pub principal: String,
    pub cost: String,
    pub discount: String,
    pub host: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_query_template_view(item: &QueryTemplate) -> QueryTemplateView {
    QueryTemplateView {
        code: item.code.clone(),
        title: item.title.clone(),
        phone_text: text::truncate(&item.phone, 80),
        principal: item.principal.clone(),
        cost: format!("{:.2}", item.cost),
        discount: format!("{:.2}", item.discount),
        host: text::truncate(&item.host, 64),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_template_views(items: &[QueryTemplate]) -> Vec<QueryTemplateView> {
    let mut views: Vec<QueryTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_template_view(item));
    }
    views
}

pub fn sum_query_template_view(view: &QueryTemplateView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_cost + parsed_discount)
}
