use super::*;

use acme_kit::text;

/// How screens and exports show a batched query slot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlotView {
    pub code: String,
    pub title_text: String,
    pub host_text: String,
    pub locale_text: String,
    pub issuer: String,
    pub rotation_days_text: String,
    pub budget: String,
    pub score: String,
    pub ratio_text: String,
    pub unit: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_batched_query_slot_view(item: &BatchedQuerySlot) -> BatchedQuerySlotView {
    BatchedQuerySlotView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        host_text: item.host.clone(),
        locale_text: text::truncate(&item.locale, 80),
        issuer: item.issuer.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        budget: format!("{:.2}", item.budget),
        score: format!("{:.2}", item.score),
        ratio_text: format!("{:.2}", item.ratio),
        unit: text::truncate(&item.unit, 64),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_query_slot_views(items: &[BatchedQuerySlot]) -> Vec<BatchedQuerySlotView> {
    let mut views: Vec<BatchedQuerySlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_query_slot_view(item));
    }
    views
}

pub fn sum_batched_query_slot_view(view: &BatchedQuerySlotView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_budget + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_query_slot("code-78", "title-361");
        let view = to_batched_query_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_query_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
