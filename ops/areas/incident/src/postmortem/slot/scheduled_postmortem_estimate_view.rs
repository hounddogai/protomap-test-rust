use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled postmortem estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateView {
    pub code_text: String,
    pub title: String,
    pub summary: String,
    pub batch_size_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_scheduled_postmortem_estimate_view(item: &ScheduledPostmortemEstimate) -> ScheduledPostmortemEstimateView {
    ScheduledPostmortemEstimateView {
        code_text: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        summary: item.summary.clone(),
        batch_size_text: item.batch_size.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_postmortem_estimate_views(
    items: &[ScheduledPostmortemEstimate],
) -> Vec<ScheduledPostmortemEstimateView> {
    let mut views: Vec<ScheduledPostmortemEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_postmortem_estimate_view(item));
    }
    views
}

pub fn sum_scheduled_postmortem_estimate_view(view: &ScheduledPostmortemEstimateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_postmortem_estimate("code-810", "title-646");
        let view = to_scheduled_postmortem_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_postmortem_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
