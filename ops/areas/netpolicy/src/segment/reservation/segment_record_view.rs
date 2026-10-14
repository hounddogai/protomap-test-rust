use super::*;

use acme_kit::text;

/// How screens and exports show a segment record.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRecordView {
    pub code: String,
    pub label_text: String,
    pub duration_minutes: String,
    pub retries: String,
    pub limit: String,
    pub burst: String,
    pub weight: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_segment_record_view(item: &SegmentRecord) -> SegmentRecordView {
    SegmentRecordView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        duration_minutes: item.duration_minutes.to_string(),
        retries: item.retries.to_string(),
        limit: item.limit.to_string(),
        burst: item.burst.to_string(),
        weight: format!("{:.2}", item.weight),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_record_views(items: &[SegmentRecord]) -> Vec<SegmentRecordView> {
    let mut views: Vec<SegmentRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_record_view(item));
    }
    views
}

pub fn sum_segment_record_view(view: &SegmentRecordView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_retries as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_record("code-72", "label-819");
        let view = to_segment_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
