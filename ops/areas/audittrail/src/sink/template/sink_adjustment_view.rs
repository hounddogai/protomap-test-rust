use super::*;

use acme_kit::text;

/// How screens and exports show a sink adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustmentView {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub ttl_seconds: String,
    pub quantity_text: String,
    pub grace_days: String,
    pub name: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_sink_adjustment_view(item: &SinkAdjustment) -> SinkAdjustmentView {
    SinkAdjustmentView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 120),
        segment: item.segment.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        quantity_text: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        name: item.name.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_adjustment_views(items: &[SinkAdjustment]) -> Vec<SinkAdjustmentView> {
    let mut views: Vec<SinkAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_adjustment_view(item));
    }
    views
}

pub fn sum_sink_adjustment_view(view: &SinkAdjustmentView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_quantity as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_adjustment("code-399", "title-173");
        let view = to_sink_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
