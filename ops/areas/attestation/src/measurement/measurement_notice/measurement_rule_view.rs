use super::*;

/// How screens and exports show a measurement rule.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRuleView {
    pub code: String,
    pub label: String,
    pub rate_text: String,
    pub discount: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_measurement_rule_view(item: &MeasurementRule) -> MeasurementRuleView {
    MeasurementRuleView {
        code: item.code.clone(),
        label: item.label.clone(),
        rate_text: format!("{:.2}", item.rate),
        discount: format!("{:.2}", item.discount),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_rule_views(items: &[MeasurementRule]) -> Vec<MeasurementRuleView> {
    let mut views: Vec<MeasurementRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_rule_view(item));
    }
    views
}

pub fn sum_measurement_rule_view(view: &MeasurementRuleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_rate + parsed_discount + parsed_maximum)
}
