use super::*;

use acme_kit::text;

/// How screens and exports show a forecast notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeView {
    pub code_text: String,
    pub label: String,
    pub timezone_text: String,
    pub title: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_forecast_notice_view(item: &ForecastNotice) -> ForecastNoticeView {
    ForecastNoticeView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        timezone_text: text::truncate(&item.timezone, 80),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_notice_views(items: &[ForecastNotice]) -> Vec<ForecastNoticeView> {
    let mut views: Vec<ForecastNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_notice_view(item));
    }
    views
}

pub fn sum_forecast_notice_view(view: &ForecastNoticeView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_notice("code-352", "label-192");
        let view = to_forecast_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
