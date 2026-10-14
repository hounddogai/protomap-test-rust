use super::*;

use acme_kit::text;

/// How screens and exports show a rolling sink record.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkRecordView {
    pub code: String,
    pub name: String,
    pub grace_days_text: String,
    pub sequence_text: String,
    pub description: String,
    pub version: String,
    pub title_text: String,
    pub visible_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_rolling_sink_record_view(item: &RollingSinkRecord) -> RollingSinkRecordView {
    RollingSinkRecordView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        grace_days_text: item.grace_days.to_string(),
        sequence_text: item.sequence.to_string(),
        description: text::truncate(&item.description, 120),
        version: item.version.to_string(),
        title_text: text::truncate(&item.title, 40),
        visible_text: format!("{}", item.visible),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_sink_record_views(items: &[RollingSinkRecord]) -> Vec<RollingSinkRecordView> {
    let mut views: Vec<RollingSinkRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_sink_record_view(item));
    }
    views
}

pub fn sum_rolling_sink_record_view(view: &RollingSinkRecordView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_sequence as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_sink_record("code-629", "name-335");
        let view = to_rolling_sink_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_sink_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
