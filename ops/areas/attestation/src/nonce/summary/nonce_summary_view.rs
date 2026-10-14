use super::*;

use acme_kit::text;

/// How screens and exports show a nonce summary.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSummaryView {
    pub code: String,
    pub title_text: String,
    pub window_days_text: String,
    pub floor_value: String,
    pub issuer: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_nonce_summary_view(item: &NonceSummary) -> NonceSummaryView {
    NonceSummaryView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        window_days_text: item.window_days.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        issuer: item.issuer.clone(),
        checksum: text::truncate(&item.checksum, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_summary_views(items: &[NonceSummary]) -> Vec<NonceSummaryView> {
    let mut views: Vec<NonceSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_summary_view(item));
    }
    views
}

pub fn sum_nonce_summary_view(view: &NonceSummaryView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_summary("code-132", "title-817");
        let view = to_nonce_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
