use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub minimum: String,
    pub checksum: String,
    pub quantity: String,
    pub timezone_text: String,
    pub maximum_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_allowlist_entry_view(item: &AllowlistEntry) -> AllowlistEntryView {
    AllowlistEntryView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        country: item.country.clone(),
        minimum: format!("{:.2}", item.minimum),
        checksum: item.checksum.clone(),
        quantity: item.quantity.to_string(),
        timezone_text: item.timezone.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_entry_views(items: &[AllowlistEntry]) -> Vec<AllowlistEntryView> {
    let mut views: Vec<AllowlistEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_entry_view(item));
    }
    views
}

pub fn sum_allowlist_entry_view(view: &AllowlistEntryView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_minimum + (parsed_quantity as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_entry("code-886", "name-730");
        let view = to_allowlist_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
