use super::*;

use acme_kit::text;

/// How screens and exports show a rolling sink mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMappingView {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub external_ref_text: String,
    pub timezone: String,
    pub threshold: String,
    pub cipher: String,
    pub cost: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_rolling_sink_mapping_view(item: &RollingSinkMapping) -> RollingSinkMappingView {
    RollingSinkMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        phone: text::truncate(&item.phone, 80),
        external_ref_text: item.external_ref.clone(),
        timezone: item.timezone.clone(),
        threshold: format!("{:.2}", item.threshold),
        cipher: item.cipher.clone(),
        cost: format!("{:.2}", item.cost),
        principal_text: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_sink_mapping_views(items: &[RollingSinkMapping]) -> Vec<RollingSinkMappingView> {
    let mut views: Vec<RollingSinkMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_sink_mapping_view(item));
    }
    views
}

pub fn sum_rolling_sink_mapping_view(view: &RollingSinkMappingView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_threshold + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_sink_mapping("code-218", "label-984");
        let view = to_rolling_sink_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_sink_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
