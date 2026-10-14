use super::*;

/// How screens and exports show a batched timeline record.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelineRecordView {
    pub code: String,
    pub name: String,
    pub discount: String,
    pub grace_days: String,
    pub priority: String,
    pub floor_value: String,
    pub budget: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_batched_timeline_record_view(item: &BatchedTimelineRecord) -> BatchedTimelineRecordView {
    BatchedTimelineRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        grace_days: item.grace_days.to_string(),
        priority: item.priority.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        budget: format!("{:.2}", item.budget),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_timeline_record_views(items: &[BatchedTimelineRecord]) -> Vec<BatchedTimelineRecordView> {
    let mut views: Vec<BatchedTimelineRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_timeline_record_view(item));
    }
    views
}

pub fn sum_batched_timeline_record_view(view: &BatchedTimelineRecordView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_discount + (parsed_grace_days as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_timeline_record("code-43", "name-457");
        let view = to_batched_timeline_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_timeline_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
