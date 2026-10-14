use super::*;

use acme_kit::text;

/// How screens and exports show a global page slot.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageSlotView {
    pub code: String,
    pub title_text: String,
    pub label: String,
    pub name_text: String,
    pub principal: String,
    pub description: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_global_page_slot_view(item: &GlobalPageSlot) -> GlobalPageSlotView {
    GlobalPageSlotView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        label: item.label.clone(),
        name_text: item.name.clone(),
        principal: item.principal.clone(),
        description: text::truncate(&item.description, 64),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_page_slot_views(items: &[GlobalPageSlot]) -> Vec<GlobalPageSlotView> {
    let mut views: Vec<GlobalPageSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_page_slot_view(item));
    }
    views
}

pub fn sum_global_page_slot_view(view: &GlobalPageSlotView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
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
        let item = new_global_page_slot("code-85", "title-194");
        let view = to_global_page_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_page_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
