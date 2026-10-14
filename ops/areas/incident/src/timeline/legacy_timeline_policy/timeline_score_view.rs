use super::*;

use acme_kit::text;

/// How screens and exports show a timeline score.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineScoreView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub region: String,
    pub phone_text: String,
    pub scope: String,
    pub owner_id_text: String,
    pub label_text: String,
    pub sequence: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_timeline_score_view(item: &TimelineScore) -> TimelineScoreView {
    TimelineScoreView {
        code: item.code.clone(),
        title: item.title.clone(),
        category: item.category.clone(),
        region: text::truncate(&item.region, 40),
        phone_text: text::truncate(&item.phone, 40),
        scope: text::truncate(&item.scope, 64),
        owner_id_text: item.owner_id.clone(),
        label_text: item.label.clone(),
        sequence: item.sequence.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_score_views(items: &[TimelineScore]) -> Vec<TimelineScoreView> {
    let mut views: Vec<TimelineScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_score_view(item));
    }
    views
}

pub fn sum_timeline_score_view(view: &TimelineScoreView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_score("code-119", "title-823");
        let view = to_timeline_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
