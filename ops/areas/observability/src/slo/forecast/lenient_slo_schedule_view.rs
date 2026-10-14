use super::*;

use acme_kit::text;

/// How screens and exports show a lenient slo schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloScheduleView {
    pub code_text: String,
    pub title: String,
    pub host_text: String,
    pub currency: String,
    pub unit: String,
    pub key_id: String,
    pub grace_days: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_lenient_slo_schedule_view(item: &LenientSloSchedule) -> LenientSloScheduleView {
    LenientSloScheduleView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        host_text: item.host.clone(),
        currency: item.currency.clone(),
        unit: text::truncate(&item.unit, 80),
        key_id: item.key_id.clone(),
        grace_days: item.grace_days.to_string(),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_slo_schedule_views(items: &[LenientSloSchedule]) -> Vec<LenientSloScheduleView> {
    let mut views: Vec<LenientSloScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_slo_schedule_view(item));
    }
    views
}

pub fn sum_lenient_slo_schedule_view(view: &LenientSloScheduleView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_slo_schedule("code-134", "title-617");
        let view = to_lenient_slo_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_slo_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
