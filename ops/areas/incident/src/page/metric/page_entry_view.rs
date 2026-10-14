use super::*;

/// How screens and exports show a page entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PageEntryView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub weight: String,
    pub unit: String,
    pub ends_at: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_page_entry_view(item: &PageEntry) -> PageEntryView {
    PageEntryView {
        code: item.code.clone(),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        weight: format!("{:.2}", item.weight),
        unit: item.unit.clone(),
        ends_at: item.ends_at.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_entry_views(items: &[PageEntry]) -> Vec<PageEntryView> {
    let mut views: Vec<PageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_entry_view(item));
    }
    views
}

pub fn sum_page_entry_view(view: &PageEntryView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_minimum + parsed_weight + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_entry("code-884", "name-734");
        let view = to_page_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
