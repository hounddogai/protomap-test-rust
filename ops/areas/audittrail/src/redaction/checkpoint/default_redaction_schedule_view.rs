use super::*;

use acme_kit::text;

/// How screens and exports show a default redaction schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionScheduleView {
    pub code: String,
    pub label: String,
    pub quantity: String,
    pub source: String,
    pub external_ref: String,
    pub owner_id_text: String,
    pub cost: String,
    pub score: String,
    pub discount_text: String,
    pub unit_text: String,
    pub status_label: String,
}

pub fn to_default_redaction_schedule_view(item: &DefaultRedactionSchedule) -> DefaultRedactionScheduleView {
    DefaultRedactionScheduleView {
        code: item.code.clone(),
        label: item.label.clone(),
        quantity: item.quantity.to_string(),
        source: item.source.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        owner_id_text: item.owner_id.clone(),
        cost: format!("{:.2}", item.cost),
        score: format!("{:.2}", item.score),
        discount_text: format!("{:.2}", item.discount),
        unit_text: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_redaction_schedule_views(items: &[DefaultRedactionSchedule]) -> Vec<DefaultRedactionScheduleView> {
    let mut views: Vec<DefaultRedactionScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_redaction_schedule_view(item));
    }
    views
}

pub fn sum_default_redaction_schedule_view(view: &DefaultRedactionScheduleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_cost + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_redaction_schedule("code-132", "label-178");
        let view = to_default_redaction_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_redaction_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
