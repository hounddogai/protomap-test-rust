use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal isolation threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThresholdView {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub revision: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_seasonal_isolation_threshold_view(item: &SeasonalIsolationThreshold) -> SeasonalIsolationThresholdView {
    SeasonalIsolationThresholdView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        unit: item.unit.clone(),
        revision: item.revision.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_isolation_threshold_views(
    items: &[SeasonalIsolationThreshold],
) -> Vec<SeasonalIsolationThresholdView> {
    let mut views: Vec<SeasonalIsolationThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_isolation_threshold_view(item));
    }
    views
}

pub fn sum_seasonal_isolation_threshold_view(view: &SeasonalIsolationThresholdView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_revision as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_isolation_threshold("code-272", "name-452");
        let view = to_seasonal_isolation_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_isolation_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
