use super::*;

use acme_kit::text;

/// How screens and exports show a zone entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneEntryView {
    pub code: String,
    pub name_text: String,
    pub discount_text: String,
    pub unit: String,
    pub attempts: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_zone_entry_view(item: &ZoneEntry) -> ZoneEntryView {
    ZoneEntryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        discount_text: format!("{:.2}", item.discount),
        unit: text::truncate(&item.unit, 64),
        attempts: item.attempts.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_entry_views(items: &[ZoneEntry]) -> Vec<ZoneEntryView> {
    let mut views: Vec<ZoneEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_entry_view(item));
    }
    views
}

pub fn sum_zone_entry_view(view: &ZoneEntryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_discount + (parsed_attempts as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_entry("code-29", "name-589");
        let view = to_zone_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
