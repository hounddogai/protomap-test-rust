use super::*;

use acme_kit::text;

/// How screens and exports show a default sink notice.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkNoticeView {
    pub code: String,
    pub name: String,
    pub retries_text: String,
    pub attempts: String,
    pub version_text: String,
    pub limit_text: String,
    pub phone: String,
    pub title: String,
    pub window_days_text: String,
    pub issuer: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_default_sink_notice_view(item: &DefaultSinkNotice) -> DefaultSinkNoticeView {
    DefaultSinkNoticeView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        retries_text: item.retries.to_string(),
        attempts: item.attempts.to_string(),
        version_text: item.version.to_string(),
        limit_text: item.limit.to_string(),
        phone: item.phone.clone(),
        title: item.title.clone(),
        window_days_text: item.window_days.to_string(),
        issuer: item.issuer.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_sink_notice_views(items: &[DefaultSinkNotice]) -> Vec<DefaultSinkNoticeView> {
    let mut views: Vec<DefaultSinkNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_sink_notice_view(item));
    }
    views
}

pub fn sum_default_sink_notice_view(view: &DefaultSinkNoticeView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_attempts as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_sink_notice("code-698", "name-228");
        let view = to_default_sink_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_sink_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
