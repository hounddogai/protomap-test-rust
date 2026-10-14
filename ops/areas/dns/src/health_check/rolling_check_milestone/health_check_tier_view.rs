use super::*;

use acme_kit::text;

/// How screens and exports show a health check tier.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTierView {
    pub code: String,
    pub name: String,
    pub rate: String,
    pub region: String,
    pub source_text: String,
    pub ttl_seconds_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_health_check_tier_view(item: &HealthCheckTier) -> HealthCheckTierView {
    HealthCheckTierView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        rate: format!("{:.2}", item.rate),
        region: text::truncate(&item.region, 40),
        source_text: item.source.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        key_id: text::truncate(&item.key_id, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_tier_views(items: &[HealthCheckTier]) -> Vec<HealthCheckTierView> {
    let mut views: Vec<HealthCheckTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_tier_view(item));
    }
    views
}

pub fn sum_health_check_tier_view(view: &HealthCheckTierView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_rate + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_tier("code-516", "name-382");
        let view = to_health_check_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
