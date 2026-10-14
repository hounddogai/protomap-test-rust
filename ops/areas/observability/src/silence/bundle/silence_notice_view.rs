use super::*;

use acme_kit::text;

/// How screens and exports show a silence notice.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceNoticeView {
    pub code: String,
    pub title: String,
    pub window_days_text: String,
    pub active_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_silence_notice_view(item: &SilenceNotice) -> SilenceNoticeView {
    SilenceNoticeView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 80),
        window_days_text: item.window_days.to_string(),
        active_text: format!("{}", item.active),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_notice_views(items: &[SilenceNotice]) -> Vec<SilenceNoticeView> {
    let mut views: Vec<SilenceNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_notice_view(item));
    }
    views
}

pub fn sum_silence_notice_view(view: &SilenceNoticeView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_notice("code-986", "title-501");
        let view = to_silence_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
