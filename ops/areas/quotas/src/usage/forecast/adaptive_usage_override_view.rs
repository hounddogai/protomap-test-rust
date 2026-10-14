use super::*;

/// How screens and exports show a adaptive usage override.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageOverrideView {
    pub code_text: String,
    pub label: String,
    pub approved: String,
    pub discount: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_adaptive_usage_override_view(item: &AdaptiveUsageOverride) -> AdaptiveUsageOverrideView {
    AdaptiveUsageOverrideView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        approved: format!("{}", item.approved),
        discount: format!("{:.2}", item.discount),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_usage_override_views(items: &[AdaptiveUsageOverride]) -> Vec<AdaptiveUsageOverrideView> {
    let mut views: Vec<AdaptiveUsageOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_usage_override_view(item));
    }
    views
}

pub fn sum_adaptive_usage_override_view(view: &AdaptiveUsageOverrideView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_discount + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_usage_override("code-854", "label-112");
        let view = to_adaptive_usage_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_usage_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
