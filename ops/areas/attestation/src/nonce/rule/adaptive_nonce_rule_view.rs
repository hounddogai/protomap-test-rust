use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive nonce rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceRuleView {
    pub code: String,
    pub title: String,
    pub channel_text: String,
    pub ttl_seconds_text: String,
    pub country_text: String,
    pub ratio: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_adaptive_nonce_rule_view(item: &AdaptiveNonceRule) -> AdaptiveNonceRuleView {
    AdaptiveNonceRuleView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        channel_text: text::truncate(&item.channel, 40),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        country_text: item.country.clone(),
        ratio: format!("{:.2}", item.ratio),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_nonce_rule_views(items: &[AdaptiveNonceRule]) -> Vec<AdaptiveNonceRuleView> {
    let mut views: Vec<AdaptiveNonceRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_nonce_rule_view(item));
    }
    views
}

pub fn sum_adaptive_nonce_rule_view(view: &AdaptiveNonceRuleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_ratio + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_nonce_rule("code-465", "title-604");
        let view = to_adaptive_nonce_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_nonce_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
