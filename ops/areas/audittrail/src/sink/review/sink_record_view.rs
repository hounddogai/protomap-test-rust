use super::*;

/// How screens and exports show a sink record.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub active_text: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_sink_record_view(item: &SinkRecord) -> SinkRecordView {
    SinkRecordView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        active_text: format!("{}", item.active),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_record_views(items: &[SinkRecord]) -> Vec<SinkRecordView> {
    let mut views: Vec<SinkRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_record_view(item));
    }
    views
}

pub fn sum_sink_record_view(view: &SinkRecordView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_record("code-687", "title-599");
        let view = to_sink_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
