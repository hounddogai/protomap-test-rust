use super::*;

use acme_kit::text;

/// How screens and exports show a weighted silence entry.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntryView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub quantity_text: String,
    pub title_text: String,
    pub threshold: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_weighted_silence_entry_view(item: &WeightedSilenceEntry) -> WeightedSilenceEntryView {
    WeightedSilenceEntryView {
        code: item.code.clone(),
        label: item.label.clone(),
        summary: text::truncate(&item.summary, 120),
        quantity_text: item.quantity.to_string(),
        title_text: text::truncate(&item.title, 120),
        threshold: format!("{:.2}", item.threshold),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_silence_entry_views(items: &[WeightedSilenceEntry]) -> Vec<WeightedSilenceEntryView> {
    let mut views: Vec<WeightedSilenceEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_silence_entry_view(item));
    }
    views
}

pub fn sum_weighted_silence_entry_view(view: &WeightedSilenceEntryView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_silence_entry("code-984", "label-539");
        let view = to_weighted_silence_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_silence_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
