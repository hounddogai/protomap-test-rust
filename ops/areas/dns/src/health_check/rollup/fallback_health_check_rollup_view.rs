use super::*;

use acme_kit::text;

/// How screens and exports show a fallback health check rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackHealthCheckRollupView {
    pub code: String,
    pub name: String,
    pub margin_text: String,
    pub maximum: String,
    pub reviewer_text: String,
    pub principal: String,
    pub minimum: String,
    pub summary_text: String,
    pub pinned: String,
    pub cluster: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_fallback_health_check_rollup_view(item: &FallbackHealthCheckRollup) -> FallbackHealthCheckRollupView {
    FallbackHealthCheckRollupView {
        code: item.code.clone(),
        name: item.name.clone(),
        margin_text: format!("{:.2}", item.margin),
        maximum: format!("{:.2}", item.maximum),
        reviewer_text: text::truncate(&item.reviewer, 64),
        principal: text::truncate(&item.principal, 80),
        minimum: format!("{:.2}", item.minimum),
        summary_text: item.summary.clone(),
        pinned: format!("{}", item.pinned),
        cluster: item.cluster.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_health_check_rollup_views(
    items: &[FallbackHealthCheckRollup],
) -> Vec<FallbackHealthCheckRollupView> {
    let mut views: Vec<FallbackHealthCheckRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_health_check_rollup_view(item));
    }
    views
}

pub fn sum_fallback_health_check_rollup_view(view: &FallbackHealthCheckRollupView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_margin + parsed_maximum + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_health_check_rollup("code-87", "name-23");
        let view = to_fallback_health_check_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_health_check_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
