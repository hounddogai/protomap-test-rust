use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard notice.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardNoticeView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub margin: String,
    pub country_text: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_dashboard_notice_view(item: &DashboardNotice) -> DashboardNoticeView {
    DashboardNoticeView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        margin: format!("{:.2}", item.margin),
        country_text: item.country.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_notice_views(items: &[DashboardNotice]) -> Vec<DashboardNoticeView> {
    let mut views: Vec<DashboardNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_notice_view(item));
    }
    views
}

pub fn sum_dashboard_notice_view(view: &DashboardNoticeView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_burst as f64 + parsed_margin + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_notice("code-496", "label-692");
        let view = to_dashboard_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
