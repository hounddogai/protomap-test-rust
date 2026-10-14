use super::*;

use acme_kit::text;

/// How screens and exports show a lenient health check rule.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientHealthCheckRuleView {
    pub code_text: String,
    pub name: String,
    pub unit: String,
    pub currency: String,
    pub window_days: String,
    pub score: String,
    pub maximum: String,
    pub port_text: String,
    pub status_label: String,
}

pub fn to_lenient_health_check_rule_view(item: &LenientHealthCheckRule) -> LenientHealthCheckRuleView {
    LenientHealthCheckRuleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        unit: text::truncate(&item.unit, 40),
        currency: text::truncate(&item.currency, 120),
        window_days: item.window_days.to_string(),
        score: format!("{:.2}", item.score),
        maximum: format!("{:.2}", item.maximum),
        port_text: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_health_check_rule_views(items: &[LenientHealthCheckRule]) -> Vec<LenientHealthCheckRuleView> {
    let mut views: Vec<LenientHealthCheckRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_health_check_rule_view(item));
    }
    views
}

pub fn sum_lenient_health_check_rule_view(view: &LenientHealthCheckRuleView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_score + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_health_check_rule("code-819", "name-153");
        let view = to_lenient_health_check_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_health_check_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
