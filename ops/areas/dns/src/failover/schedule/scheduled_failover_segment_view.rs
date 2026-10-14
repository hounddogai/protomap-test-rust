use super::*;

/// How screens and exports show a scheduled failover segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegmentView {
    pub code_text: String,
    pub title_text: String,
    pub score: String,
    pub window_days_text: String,
    pub retries_text: String,
    pub timezone: String,
    pub segment: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_scheduled_failover_segment_view(item: &ScheduledFailoverSegment) -> ScheduledFailoverSegmentView {
    ScheduledFailoverSegmentView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        score: format!("{:.2}", item.score),
        window_days_text: item.window_days.to_string(),
        retries_text: item.retries.to_string(),
        timezone: item.timezone.clone(),
        segment: item.segment.clone(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_failover_segment_views(items: &[ScheduledFailoverSegment]) -> Vec<ScheduledFailoverSegmentView> {
    let mut views: Vec<ScheduledFailoverSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_failover_segment_view(item));
    }
    views
}

pub fn sum_scheduled_failover_segment_view(view: &ScheduledFailoverSegmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_score + (parsed_window_days as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_failover_segment("code-414", "title-62");
        let view = to_scheduled_failover_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_failover_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
