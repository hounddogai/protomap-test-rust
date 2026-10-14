use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled zone score.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneScoreView {
    pub code: String,
    pub name: String,
    pub weight: String,
    pub segment: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_scheduled_zone_score_view(item: &ScheduledZoneScore) -> ScheduledZoneScoreView {
    ScheduledZoneScoreView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        segment: item.segment.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_zone_score_views(items: &[ScheduledZoneScore]) -> Vec<ScheduledZoneScoreView> {
    let mut views: Vec<ScheduledZoneScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_zone_score_view(item));
    }
    views
}

pub fn sum_scheduled_zone_score_view(view: &ScheduledZoneScoreView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_weight + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_zone_score("code-894", "name-21");
        let view = to_scheduled_zone_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_zone_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
