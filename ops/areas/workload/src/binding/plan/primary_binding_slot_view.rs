use super::*;

use acme_kit::text;

/// How screens and exports show a primary binding slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotView {
    pub code_text: String,
    pub name: String,
    pub region: String,
    pub summary: String,
    pub window_days: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_primary_binding_slot_view(item: &PrimaryBindingSlot) -> PrimaryBindingSlotView {
    PrimaryBindingSlotView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        region: item.region.clone(),
        summary: item.summary.clone(),
        window_days: item.window_days.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_binding_slot_views(items: &[PrimaryBindingSlot]) -> Vec<PrimaryBindingSlotView> {
    let mut views: Vec<PrimaryBindingSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_binding_slot_view(item));
    }
    views
}

pub fn sum_primary_binding_slot_view(view: &PrimaryBindingSlotView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_binding_slot("code-761", "name-474");
        let view = to_primary_binding_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_binding_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
