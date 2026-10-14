use super::*;

use acme_kit::text;

/// How screens and exports show a rolling egress entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressEntryView {
    pub code_text: String,
    pub label_text: String,
    pub rate: String,
    pub source: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_rolling_egress_entry_view(item: &RollingEgressEntry) -> RollingEgressEntryView {
    RollingEgressEntryView {
        code_text: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        source: text::truncate(&item.source, 64),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_egress_entry_views(items: &[RollingEgressEntry]) -> Vec<RollingEgressEntryView> {
    let mut views: Vec<RollingEgressEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_egress_entry_view(item));
    }
    views
}

pub fn sum_rolling_egress_entry_view(view: &RollingEgressEntryView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_rate + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_egress_entry("code-129", "label-972");
        let view = to_rolling_egress_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_egress_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
