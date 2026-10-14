use super::*;

/// How screens and exports show a legacy forecast bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyForecastBundleView {
    pub code: String,
    pub label: String,
    pub revision: String,
    pub capacity: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_legacy_forecast_bundle_view(item: &LegacyForecastBundle) -> LegacyForecastBundleView {
    LegacyForecastBundleView {
        code: item.code.clone(),
        label: item.label.clone(),
        revision: item.revision.to_string(),
        capacity: item.capacity.to_string(),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_forecast_bundle_views(items: &[LegacyForecastBundle]) -> Vec<LegacyForecastBundleView> {
    let mut views: Vec<LegacyForecastBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_forecast_bundle_view(item));
    }
    views
}

pub fn sum_legacy_forecast_bundle_view(view: &LegacyForecastBundleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_forecast_bundle("code-146", "label-267");
        let view = to_legacy_forecast_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_forecast_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
