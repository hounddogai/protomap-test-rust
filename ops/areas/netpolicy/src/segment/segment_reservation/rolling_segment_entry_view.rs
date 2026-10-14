use super::*;

use acme_kit::text;

/// How screens and exports show a rolling segment entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentEntryView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub label: String,
    pub amount_text: String,
    pub notes_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_rolling_segment_entry_view(item: &RollingSegmentEntry) -> RollingSegmentEntryView {
    RollingSegmentEntryView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        region: item.region.clone(),
        label: item.label.clone(),
        amount_text: format!("{:.2}", item.amount),
        notes_text: text::truncate(&item.notes, 40),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_segment_entry_views(items: &[RollingSegmentEntry]) -> Vec<RollingSegmentEntryView> {
    let mut views: Vec<RollingSegmentEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_segment_entry_view(item));
    }
    views
}

pub fn sum_rolling_segment_entry_view(view: &RollingSegmentEntryView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_segment_entry("code-647", "name-424");
        let view = to_rolling_segment_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_segment_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
