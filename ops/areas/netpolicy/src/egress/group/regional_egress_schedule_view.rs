use super::*;

use acme_kit::text;

/// How screens and exports show a regional egress schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressScheduleView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub fingerprint_text: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_regional_egress_schedule_view(item: &RegionalEgressSchedule) -> RegionalEgressScheduleView {
    RegionalEgressScheduleView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        archived: format!("{}", item.archived),
        fingerprint_text: item.fingerprint.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_egress_schedule_views(items: &[RegionalEgressSchedule]) -> Vec<RegionalEgressScheduleView> {
    let mut views: Vec<RegionalEgressScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_egress_schedule_view(item));
    }
    views
}

pub fn sum_regional_egress_schedule_view(view: &RegionalEgressScheduleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_egress_schedule("code-192", "title-901");
        let view = to_regional_egress_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_egress_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
