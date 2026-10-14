use super::*;

/// How screens and exports show a manual health check baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBaselineView {
    pub code_text: String,
    pub name: String,
    pub minimum: String,
    pub maximum: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_manual_health_check_baseline_view(item: &ManualHealthCheckBaseline) -> ManualHealthCheckBaselineView {
    ManualHealthCheckBaselineView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        maximum: format!("{:.2}", item.maximum),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_health_check_baseline_views(
    items: &[ManualHealthCheckBaseline],
) -> Vec<ManualHealthCheckBaselineView> {
    let mut views: Vec<ManualHealthCheckBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_health_check_baseline_view(item));
    }
    views
}

pub fn sum_manual_health_check_baseline_view(view: &ManualHealthCheckBaselineView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_minimum + parsed_maximum + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_health_check_baseline("code-370", "name-432");
        let view = to_manual_health_check_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_health_check_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
