use super::*;

use acme_kit::text;

/// How screens and exports show a failover adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAdjustmentView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub visible_text: String,
    pub sequence_text: String,
    pub external_ref: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_failover_adjustment_view(item: &FailoverAdjustment) -> FailoverAdjustmentView {
    FailoverAdjustmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        currency: text::truncate(&item.currency, 64),
        visible_text: format!("{}", item.visible),
        sequence_text: item.sequence.to_string(),
        external_ref: text::truncate(&item.external_ref, 64),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_adjustment_views(items: &[FailoverAdjustment]) -> Vec<FailoverAdjustmentView> {
    let mut views: Vec<FailoverAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_adjustment_view(item));
    }
    views
}

pub fn sum_failover_adjustment_view(view: &FailoverAdjustmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_adjustment("code-266", "label-522");
        let view = to_failover_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
