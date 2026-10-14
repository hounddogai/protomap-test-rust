use super::*;

use acme_kit::text;

/// How screens and exports show a capped page summary.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageSummaryView {
    pub code: String,
    pub label_text: String,
    pub quota_units_text: String,
    pub summary: String,
    pub window_days_text: String,
    pub scope: String,
    pub tenant_id_text: String,
    pub segment_text: String,
    pub rate: String,
    pub created_at: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_capped_page_summary_view(item: &CappedPageSummary) -> CappedPageSummaryView {
    CappedPageSummaryView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        quota_units_text: item.quota_units.to_string(),
        summary: item.summary.clone(),
        window_days_text: item.window_days.to_string(),
        scope: item.scope.clone(),
        tenant_id_text: item.tenant_id.clone(),
        segment_text: text::truncate(&item.segment, 120),
        rate: format!("{:.2}", item.rate),
        created_at: item.created_at.to_string(),
        contact_email: text::truncate(&item.contact_email, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_page_summary_views(items: &[CappedPageSummary]) -> Vec<CappedPageSummaryView> {
    let mut views: Vec<CappedPageSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_page_summary_view(item));
    }
    views
}

pub fn sum_capped_page_summary_view(view: &CappedPageSummaryView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_window_days as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_page_summary("code-730", "label-640");
        let view = to_capped_page_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_page_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
