use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal egress schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressScheduleView {
    pub code: String,
    pub name: String,
    pub visible_text: String,
    pub minimum: String,
    pub approved: String,
    pub timezone: String,
    pub source: String,
    pub sequence_text: String,
    pub currency_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_seasonal_egress_schedule_view(item: &SeasonalEgressSchedule) -> SeasonalEgressScheduleView {
    SeasonalEgressScheduleView {
        code: item.code.clone(),
        name: item.name.clone(),
        visible_text: format!("{}", item.visible),
        minimum: format!("{:.2}", item.minimum),
        approved: format!("{}", item.approved),
        timezone: item.timezone.clone(),
        source: text::truncate(&item.source, 40),
        sequence_text: item.sequence.to_string(),
        currency_text: text::truncate(&item.currency, 64),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_egress_schedule_views(items: &[SeasonalEgressSchedule]) -> Vec<SeasonalEgressScheduleView> {
    let mut views: Vec<SeasonalEgressScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_egress_schedule_view(item));
    }
    views
}

pub fn sum_seasonal_egress_schedule_view(view: &SeasonalEgressScheduleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_minimum + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_egress_schedule("code-683", "name-115");
        let view = to_seasonal_egress_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_egress_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
