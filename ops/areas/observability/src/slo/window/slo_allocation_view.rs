use super::*;

use acme_kit::text;

/// How screens and exports show a slo allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAllocationView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub burst: String,
    pub locale: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_slo_allocation_view(item: &SloAllocation) -> SloAllocationView {
    SloAllocationView {
        code: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        burst: item.burst.to_string(),
        locale: text::truncate(&item.locale, 80),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_allocation_views(items: &[SloAllocation]) -> Vec<SloAllocationView> {
    let mut views: Vec<SloAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_allocation_view(item));
    }
    views
}

pub fn sum_slo_allocation_view(view: &SloAllocationView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_burst as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_allocation("code-802", "title-943");
        let view = to_slo_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
