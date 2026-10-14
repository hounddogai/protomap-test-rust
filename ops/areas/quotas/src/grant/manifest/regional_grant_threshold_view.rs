use super::*;

use acme_kit::text;

/// How screens and exports show a regional grant threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantThresholdView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub timezone: String,
    pub burst: String,
    pub capacity: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_regional_grant_threshold_view(item: &RegionalGrantThreshold) -> RegionalGrantThresholdView {
    RegionalGrantThresholdView {
        code: item.code.clone(),
        name: item.name.clone(),
        country: text::truncate(&item.country, 64),
        timezone: item.timezone.clone(),
        burst: item.burst.to_string(),
        capacity: item.capacity.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_grant_threshold_views(items: &[RegionalGrantThreshold]) -> Vec<RegionalGrantThresholdView> {
    let mut views: Vec<RegionalGrantThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_grant_threshold_view(item));
    }
    views
}

pub fn sum_regional_grant_threshold_view(view: &RegionalGrantThresholdView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_capacity as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_grant_threshold("code-172", "name-452");
        let view = to_regional_grant_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_grant_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
