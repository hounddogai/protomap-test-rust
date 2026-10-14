use super::*;

/// How screens and exports show a rolling slo batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloBatchView {
    pub code: String,
    pub name: String,
    pub quota_units: String,
    pub summary: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_rolling_slo_batch_view(item: &RollingSloBatch) -> RollingSloBatchView {
    RollingSloBatchView {
        code: item.code.clone(),
        name: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        summary: item.summary.clone(),
        title_text: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_slo_batch_views(items: &[RollingSloBatch]) -> Vec<RollingSloBatchView> {
    let mut views: Vec<RollingSloBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_slo_batch_view(item));
    }
    views
}

pub fn sum_rolling_slo_batch_view(view: &RollingSloBatchView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_quota_units as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_slo_batch("code-319", "name-591");
        let view = to_rolling_slo_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_slo_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
