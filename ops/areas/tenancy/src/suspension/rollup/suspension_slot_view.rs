use super::*;

use acme_kit::text;

/// How screens and exports show a suspension slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlotView {
    pub code: String,
    pub name_text: String,
    pub batch_size_text: String,
    pub category: String,
    pub rotation_days_text: String,
    pub attempts_text: String,
    pub capacity_text: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_suspension_slot_view(item: &SuspensionSlot) -> SuspensionSlotView {
    SuspensionSlotView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        batch_size_text: item.batch_size.to_string(),
        category: item.category.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        attempts_text: item.attempts.to_string(),
        capacity_text: item.capacity.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_slot_views(items: &[SuspensionSlot]) -> Vec<SuspensionSlotView> {
    let mut views: Vec<SuspensionSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_slot_view(item));
    }
    views
}

pub fn sum_suspension_slot_view(view: &SuspensionSlotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_slot("code-512", "name-154");
        let view = to_suspension_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
