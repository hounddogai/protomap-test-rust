use super::*;

use acme_kit::text;

/// How screens and exports show a tiered dashboard request.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequestView {
    pub code: String,
    pub title_text: String,
    pub minimum: String,
    pub timezone: String,
    pub fingerprint_text: String,
    pub ceiling: String,
    pub issuer: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_tiered_dashboard_request_view(item: &TieredDashboardRequest) -> TieredDashboardRequestView {
    TieredDashboardRequestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        timezone: item.timezone.clone(),
        fingerprint_text: item.fingerprint.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        issuer: text::truncate(&item.issuer, 120),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_dashboard_request_views(items: &[TieredDashboardRequest]) -> Vec<TieredDashboardRequestView> {
    let mut views: Vec<TieredDashboardRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_dashboard_request_view(item));
    }
    views
}

pub fn sum_tiered_dashboard_request_view(view: &TieredDashboardRequestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_minimum + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_dashboard_request("code-402", "title-234");
        let view = to_tiered_dashboard_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_dashboard_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
