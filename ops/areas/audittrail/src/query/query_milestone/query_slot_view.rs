use super::*;

use acme_kit::text;

/// How screens and exports show a query slot.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySlotView {
    pub code: String,
    pub title_text: String,
    pub retries: String,
    pub starts_at: String,
    pub timezone: String,
    pub scope: String,
    pub audience: String,
    pub source_text: String,
    pub quantity_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_query_slot_view(item: &QuerySlot) -> QuerySlotView {
    QuerySlotView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        retries: item.retries.to_string(),
        starts_at: item.starts_at.to_string(),
        timezone: item.timezone.clone(),
        scope: text::truncate(&item.scope, 40),
        audience: item.audience.clone(),
        source_text: item.source.clone(),
        quantity_text: item.quantity.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_slot_views(items: &[QuerySlot]) -> Vec<QuerySlotView> {
    let mut views: Vec<QuerySlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_slot_view(item));
    }
    views
}

pub fn sum_query_slot_view(view: &QuerySlotView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_starts_at as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_slot("code-579", "title-140");
        let view = to_query_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
