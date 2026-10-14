use super::*;

use acme_kit::text;

/// How screens and exports show a primary slo segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegmentView {
    pub code_text: String,
    pub title: String,
    pub priority: String,
    pub duration_minutes: String,
    pub timezone_text: String,
    pub channel: String,
    pub host_text: String,
    pub issuer_text: String,
    pub fingerprint_text: String,
    pub minimum: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_primary_slo_segment_view(item: &PrimarySloSegment) -> PrimarySloSegmentView {
    PrimarySloSegmentView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 64),
        priority: item.priority.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        timezone_text: item.timezone.clone(),
        channel: item.channel.clone(),
        host_text: item.host.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        fingerprint_text: item.fingerprint.clone(),
        minimum: format!("{:.2}", item.minimum),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_slo_segment_views(items: &[PrimarySloSegment]) -> Vec<PrimarySloSegmentView> {
    let mut views: Vec<PrimarySloSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_slo_segment_view(item));
    }
    views
}

pub fn sum_primary_slo_segment_view(view: &PrimarySloSegmentView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_duration_minutes as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_slo_segment("code-290", "title-315");
        let view = to_primary_slo_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_slo_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
