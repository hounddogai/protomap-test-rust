use super::*;

use acme_kit::text;

/// How screens and exports show a slo target.
#[derive(Debug, Clone, PartialEq)]
pub struct SloTargetView {
    pub code_text: String,
    pub label_text: String,
    pub grace_days: String,
    pub quota_units: String,
    pub fingerprint_text: String,
    pub ceiling: String,
    pub channel: String,
    pub reviewer: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_slo_target_view(item: &SloTarget) -> SloTargetView {
    SloTargetView {
        code_text: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        grace_days: item.grace_days.to_string(),
        quota_units: item.quota_units.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        channel: item.channel.clone(),
        reviewer: item.reviewer.clone(),
        source: text::truncate(&item.source, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_target_views(items: &[SloTarget]) -> Vec<SloTargetView> {
    let mut views: Vec<SloTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_target_view(item));
    }
    views
}

pub fn sum_slo_target_view(view: &SloTargetView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_quota_units as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_target("code-488", "label-870");
        let view = to_slo_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
