use super::*;

/// How screens and exports show a verifier metric.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetricView {
    pub code_text: String,
    pub name: String,
    pub cost: String,
    pub floor_value_text: String,
    pub updated_at_text: String,
    pub burst: String,
    pub version_text: String,
    pub fingerprint_text: String,
    pub category_text: String,
    pub issuer: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_verifier_metric_view(item: &VerifierMetric) -> VerifierMetricView {
    VerifierMetricView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        floor_value_text: format!("{:.2}", item.floor_value),
        updated_at_text: item.updated_at.to_string(),
        burst: item.burst.to_string(),
        version_text: item.version.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        category_text: item.category.clone(),
        issuer: item.issuer.clone(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_metric_views(items: &[VerifierMetric]) -> Vec<VerifierMetricView> {
    let mut views: Vec<VerifierMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_metric_view(item));
    }
    views
}

pub fn sum_verifier_metric_view(view: &VerifierMetricView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_cost + parsed_floor_value + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_metric("code-157", "name-113");
        let view = to_verifier_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
