use super::*;

use acme_kit::text;

/// How screens and exports show a legacy flow log override.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogOverrideView {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub region: String,
    pub cost: String,
    pub fingerprint_text: String,
    pub ratio: String,
    pub namespace_text: String,
    pub unit: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_legacy_flow_log_override_view(item: &LegacyFlowLogOverride) -> LegacyFlowLogOverrideView {
    LegacyFlowLogOverrideView {
        code: item.code.clone(),
        title: item.title.clone(),
        audience: item.audience.clone(),
        region: text::truncate(&item.region, 80),
        cost: format!("{:.2}", item.cost),
        fingerprint_text: item.fingerprint.clone(),
        ratio: format!("{:.2}", item.ratio),
        namespace_text: item.namespace.clone(),
        unit: item.unit.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_flow_log_override_views(items: &[LegacyFlowLogOverride]) -> Vec<LegacyFlowLogOverrideView> {
    let mut views: Vec<LegacyFlowLogOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_flow_log_override_view(item));
    }
    views
}

pub fn sum_legacy_flow_log_override_view(view: &LegacyFlowLogOverrideView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_cost + parsed_ratio + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_flow_log_override("code-236", "title-328");
        let view = to_legacy_flow_log_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_flow_log_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
