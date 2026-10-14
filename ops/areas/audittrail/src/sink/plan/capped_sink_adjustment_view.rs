use super::*;

/// How screens and exports show a capped sink adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkAdjustmentView {
    pub code_text: String,
    pub label: String,
    pub segment: String,
    pub source: String,
    pub minimum: String,
    pub title_text: String,
    pub timezone_text: String,
    pub attempts: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_capped_sink_adjustment_view(item: &CappedSinkAdjustment) -> CappedSinkAdjustmentView {
    CappedSinkAdjustmentView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        segment: item.segment.clone(),
        source: item.source.clone(),
        minimum: format!("{:.2}", item.minimum),
        title_text: item.title.clone(),
        timezone_text: item.timezone.clone(),
        attempts: item.attempts.to_string(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_sink_adjustment_views(items: &[CappedSinkAdjustment]) -> Vec<CappedSinkAdjustmentView> {
    let mut views: Vec<CappedSinkAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_sink_adjustment_view(item));
    }
    views
}

pub fn sum_capped_sink_adjustment_view(view: &CappedSinkAdjustmentView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_minimum + (parsed_attempts as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_sink_adjustment("code-377", "label-151");
        let view = to_capped_sink_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_sink_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
