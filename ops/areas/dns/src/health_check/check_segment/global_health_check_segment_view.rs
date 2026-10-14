use super::*;

use acme_kit::text;

/// How screens and exports show a global health check segment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalHealthCheckSegmentView {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub ttl_seconds: String,
    pub margin_text: String,
    pub currency: String,
    pub active: String,
    pub summary_text: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_global_health_check_segment_view(item: &GlobalHealthCheckSegment) -> GlobalHealthCheckSegmentView {
    GlobalHealthCheckSegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        checksum: text::truncate(&item.checksum, 64),
        ttl_seconds: item.ttl_seconds.to_string(),
        margin_text: format!("{:.2}", item.margin),
        currency: text::truncate(&item.currency, 80),
        active: format!("{}", item.active),
        summary_text: text::truncate(&item.summary, 64),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_health_check_segment_views(items: &[GlobalHealthCheckSegment]) -> Vec<GlobalHealthCheckSegmentView> {
    let mut views: Vec<GlobalHealthCheckSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_health_check_segment_view(item));
    }
    views
}

pub fn sum_global_health_check_segment_view(view: &GlobalHealthCheckSegmentView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_health_check_segment("code-192", "name-27");
        let view = to_global_health_check_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_health_check_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
