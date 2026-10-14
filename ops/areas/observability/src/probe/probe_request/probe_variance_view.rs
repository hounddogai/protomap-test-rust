use super::*;

use acme_kit::text;

/// How screens and exports show a probe variance.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeVarianceView {
    pub code: String,
    pub label: String,
    pub budget_text: String,
    pub pinned_text: String,
    pub port: String,
    pub issuer_text: String,
    pub ttl_seconds_text: String,
    pub owner_id_text: String,
    pub quantity: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_probe_variance_view(item: &ProbeVariance) -> ProbeVarianceView {
    ProbeVarianceView {
        code: item.code.clone(),
        label: item.label.clone(),
        budget_text: format!("{:.2}", item.budget),
        pinned_text: format!("{}", item.pinned),
        port: item.port.to_string(),
        issuer_text: item.issuer.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        owner_id_text: item.owner_id.clone(),
        quantity: item.quantity.to_string(),
        checksum_text: text::truncate(&item.checksum, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_variance_views(items: &[ProbeVariance]) -> Vec<ProbeVarianceView> {
    let mut views: Vec<ProbeVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_variance_view(item));
    }
    views
}

pub fn sum_probe_variance_view(view: &ProbeVarianceView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_budget + (parsed_port as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_variance("code-738", "label-324");
        let view = to_probe_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
