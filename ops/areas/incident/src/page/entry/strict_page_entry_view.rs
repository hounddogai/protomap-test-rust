use super::*;

use acme_kit::text;

/// How screens and exports show a strict page entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageEntryView {
    pub code_text: String,
    pub name_text: String,
    pub currency: String,
    pub archived: String,
    pub category: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_strict_page_entry_view(item: &StrictPageEntry) -> StrictPageEntryView {
    StrictPageEntryView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        currency: text::truncate(&item.currency, 120),
        archived: format!("{}", item.archived),
        category: item.category.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_page_entry_views(items: &[StrictPageEntry]) -> Vec<StrictPageEntryView> {
    let mut views: Vec<StrictPageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_page_entry_view(item));
    }
    views
}

pub fn sum_strict_page_entry_view(view: &StrictPageEntryView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_page_entry("code-173", "name-414");
        let view = to_strict_page_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_page_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
