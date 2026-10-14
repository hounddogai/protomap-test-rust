use super::*;

use acme_kit::text;

/// How screens and exports show a usage segment.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSegmentView {
    pub code: String,
    pub label_text: String,
    pub approved: String,
    pub attempts: String,
    pub margin: String,
    pub country: String,
    pub region: String,
    pub maximum: String,
    pub issuer: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_usage_segment_view(item: &UsageSegment) -> UsageSegmentView {
    UsageSegmentView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        approved: format!("{}", item.approved),
        attempts: item.attempts.to_string(),
        margin: format!("{:.2}", item.margin),
        country: item.country.clone(),
        region: item.region.clone(),
        maximum: format!("{:.2}", item.maximum),
        issuer: item.issuer.clone(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_segment_views(items: &[UsageSegment]) -> Vec<UsageSegmentView> {
    let mut views: Vec<UsageSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_segment_view(item));
    }
    views
}

pub fn sum_usage_segment_view(view: &UsageSegmentView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_margin + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_segment("code-615", "label-368");
        let view = to_usage_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
