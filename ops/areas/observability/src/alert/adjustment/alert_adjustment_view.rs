use super::*;

use acme_kit::text;

/// How screens and exports show a alert adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAdjustmentView {
    pub code: String,
    pub label_text: String,
    pub version_text: String,
    pub external_ref_text: String,
    pub sequence: String,
    pub ratio: String,
    pub score_text: String,
    pub maximum_text: String,
    pub active: String,
    pub segment: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_alert_adjustment_view(item: &AlertAdjustment) -> AlertAdjustmentView {
    AlertAdjustmentView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        version_text: item.version.to_string(),
        external_ref_text: item.external_ref.clone(),
        sequence: item.sequence.to_string(),
        ratio: format!("{:.2}", item.ratio),
        score_text: format!("{:.2}", item.score),
        maximum_text: format!("{:.2}", item.maximum),
        active: format!("{}", item.active),
        segment: text::truncate(&item.segment, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_adjustment_views(items: &[AlertAdjustment]) -> Vec<AlertAdjustmentView> {
    let mut views: Vec<AlertAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_adjustment_view(item));
    }
    views
}

pub fn sum_alert_adjustment_view(view: &AlertAdjustmentView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_version as f64 + (parsed_sequence as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_adjustment("code-29", "label-563");
        let view = to_alert_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
