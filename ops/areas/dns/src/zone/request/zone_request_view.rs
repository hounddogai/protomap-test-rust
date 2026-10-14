use super::*;

/// How screens and exports show a zone request.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRequestView {
    pub code: String,
    pub title_text: String,
    pub score_text: String,
    pub amount: String,
    pub discount: String,
    pub weight: String,
    pub source: String,
    pub checksum_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_zone_request_view(item: &ZoneRequest) -> ZoneRequestView {
    ZoneRequestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        score_text: format!("{:.2}", item.score),
        amount: format!("{:.2}", item.amount),
        discount: format!("{:.2}", item.discount),
        weight: format!("{:.2}", item.weight),
        source: item.source.clone(),
        checksum_text: item.checksum.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_request_views(items: &[ZoneRequest]) -> Vec<ZoneRequestView> {
    let mut views: Vec<ZoneRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_request_view(item));
    }
    views
}

pub fn sum_zone_request_view(view: &ZoneRequestView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_score + parsed_amount + parsed_discount)
}
