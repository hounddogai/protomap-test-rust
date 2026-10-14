use super::*;

use acme_kit::text;

/// How screens and exports show a probe estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEstimateView {
    pub code_text: String,
    pub name: String,
    pub rate: String,
    pub channel_text: String,
    pub floor_value_text: String,
    pub ratio_text: String,
    pub checksum: String,
    pub tenant_id: String,
    pub attempts_text: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_probe_estimate_view(item: &ProbeEstimate) -> ProbeEstimateView {
    ProbeEstimateView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        channel_text: item.channel.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        ratio_text: format!("{:.2}", item.ratio),
        checksum: text::truncate(&item.checksum, 40),
        tenant_id: item.tenant_id.clone(),
        attempts_text: item.attempts.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_estimate_views(items: &[ProbeEstimate]) -> Vec<ProbeEstimateView> {
    let mut views: Vec<ProbeEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_estimate_view(item));
    }
    views
}

pub fn sum_probe_estimate_view(view: &ProbeEstimateView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_rate + parsed_floor_value + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_estimate("code-820", "name-270");
        let view = to_probe_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
