use super::*;

use acme_kit::text;

/// How screens and exports show a slo notice.
#[derive(Debug, Clone, PartialEq)]
pub struct SloNoticeView {
    pub code_text: String,
    pub title: String,
    pub ttl_seconds: String,
    pub weight: String,
    pub fingerprint_text: String,
    pub region: String,
    pub host: String,
    pub audience_text: String,
    pub batch_size: String,
    pub reviewer: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_slo_notice_view(item: &SloNotice) -> SloNoticeView {
    SloNoticeView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        weight: format!("{:.2}", item.weight),
        fingerprint_text: item.fingerprint.clone(),
        region: item.region.clone(),
        host: text::truncate(&item.host, 64),
        audience_text: text::truncate(&item.audience, 120),
        batch_size: item.batch_size.to_string(),
        reviewer: item.reviewer.clone(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_notice_views(items: &[SloNotice]) -> Vec<SloNoticeView> {
    let mut views: Vec<SloNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_notice_view(item));
    }
    views
}

pub fn sum_slo_notice_view(view: &SloNoticeView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_weight + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_notice("code-952", "title-120");
        let view = to_slo_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
