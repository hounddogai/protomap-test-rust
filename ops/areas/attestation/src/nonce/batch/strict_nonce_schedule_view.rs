use super::*;

use acme_kit::text;

/// How screens and exports show a strict nonce schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictNonceScheduleView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub budget: String,
    pub visible: String,
    pub cluster_text: String,
    pub pinned: String,
    pub quantity_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_strict_nonce_schedule_view(item: &StrictNonceSchedule) -> StrictNonceScheduleView {
    StrictNonceScheduleView {
        code: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        budget: format!("{:.2}", item.budget),
        visible: format!("{}", item.visible),
        cluster_text: text::truncate(&item.cluster, 40),
        pinned: format!("{}", item.pinned),
        quantity_text: item.quantity.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_nonce_schedule_views(items: &[StrictNonceSchedule]) -> Vec<StrictNonceScheduleView> {
    let mut views: Vec<StrictNonceScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_nonce_schedule_view(item));
    }
    views
}

pub fn sum_strict_nonce_schedule_view(view: &StrictNonceScheduleView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_budget + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_nonce_schedule("code-181", "title-968");
        let view = to_strict_nonce_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_nonce_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
