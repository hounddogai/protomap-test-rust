use super::*;

use acme_kit::text;

/// How screens and exports show a timeline record.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRecordView {
    pub code_text: String,
    pub name: String,
    pub port: String,
    pub summary: String,
    pub locale: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_timeline_record_view(item: &TimelineRecord) -> TimelineRecordView {
    TimelineRecordView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        port: item.port.to_string(),
        summary: item.summary.clone(),
        locale: text::truncate(&item.locale, 40),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_record_views(items: &[TimelineRecord]) -> Vec<TimelineRecordView> {
    let mut views: Vec<TimelineRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_record_view(item));
    }
    views
}

pub fn sum_timeline_record_view(view: &TimelineRecordView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_port as f64 + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_record("code-604", "name-979");
        let view = to_timeline_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
