use super::*;

use acme_kit::text;

/// How screens and exports show a regional overage entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageEntryView {
    pub code: String,
    pub title_text: String,
    pub discount: String,
    pub summary_text: String,
    pub reviewer_text: String,
    pub minimum_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_regional_overage_entry_view(item: &RegionalOverageEntry) -> RegionalOverageEntryView {
    RegionalOverageEntryView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        discount: format!("{:.2}", item.discount),
        summary_text: text::truncate(&item.summary, 80),
        reviewer_text: item.reviewer.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        audience: text::truncate(&item.audience, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_overage_entry_views(items: &[RegionalOverageEntry]) -> Vec<RegionalOverageEntryView> {
    let mut views: Vec<RegionalOverageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_overage_entry_view(item));
    }
    views
}

pub fn sum_regional_overage_entry_view(view: &RegionalOverageEntryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_discount + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_overage_entry("code-367", "title-449");
        let view = to_regional_overage_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_overage_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
