use super::*;

/// How screens and exports show a manual silence slot.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceSlotView {
    pub code: String,
    pub name_text: String,
    pub archived: String,
    pub phone_text: String,
    pub limit_text: String,
    pub rotation_days_text: String,
    pub capacity: String,
    pub approved: String,
    pub cluster: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_manual_silence_slot_view(item: &ManualSilenceSlot) -> ManualSilenceSlotView {
    ManualSilenceSlotView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        archived: format!("{}", item.archived),
        phone_text: item.phone.clone(),
        limit_text: item.limit.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        capacity: item.capacity.to_string(),
        approved: format!("{}", item.approved),
        cluster: item.cluster.clone(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_silence_slot_views(items: &[ManualSilenceSlot]) -> Vec<ManualSilenceSlotView> {
    let mut views: Vec<ManualSilenceSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_silence_slot_view(item));
    }
    views
}

pub fn sum_manual_silence_slot_view(view: &ManualSilenceSlotView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_rotation_days as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_silence_slot("code-167", "name-45");
        let view = to_manual_silence_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_silence_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
