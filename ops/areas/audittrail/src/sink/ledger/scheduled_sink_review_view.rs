use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled sink review.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewView {
    pub code: String,
    pub title: String,
    pub version: String,
    pub pinned_text: String,
    pub updated_at: String,
    pub phone_text: String,
    pub description: String,
    pub rotation_days: String,
    pub notes: String,
    pub attempts: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_scheduled_sink_review_view(item: &ScheduledSinkReview) -> ScheduledSinkReviewView {
    ScheduledSinkReviewView {
        code: item.code.clone(),
        title: item.title.clone(),
        version: item.version.to_string(),
        pinned_text: format!("{}", item.pinned),
        updated_at: item.updated_at.to_string(),
        phone_text: text::truncate(&item.phone, 64),
        description: text::truncate(&item.description, 120),
        rotation_days: item.rotation_days.to_string(),
        notes: item.notes.clone(),
        attempts: item.attempts.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_sink_review_views(items: &[ScheduledSinkReview]) -> Vec<ScheduledSinkReviewView> {
    let mut views: Vec<ScheduledSinkReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_sink_review_view(item));
    }
    views
}

pub fn sum_scheduled_sink_review_view(view: &ScheduledSinkReviewView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_version as f64 + (parsed_updated_at as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_sink_review("code-600", "title-883");
        let view = to_scheduled_sink_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_sink_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
