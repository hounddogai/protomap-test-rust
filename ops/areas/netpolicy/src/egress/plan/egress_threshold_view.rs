use super::*;

use acme_kit::text;

/// How screens and exports show a egress threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressThresholdView {
    pub code_text: String,
    pub title: String,
    pub cluster: String,
    pub maximum: String,
    pub sequence: String,
    pub cipher: String,
    pub floor_value: String,
    pub window_days: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_egress_threshold_view(item: &EgressThreshold) -> EgressThresholdView {
    EgressThresholdView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cluster: item.cluster.clone(),
        maximum: format!("{:.2}", item.maximum),
        sequence: item.sequence.to_string(),
        cipher: text::truncate(&item.cipher, 64),
        floor_value: format!("{:.2}", item.floor_value),
        window_days: item.window_days.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_threshold_views(items: &[EgressThreshold]) -> Vec<EgressThresholdView> {
    let mut views: Vec<EgressThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_threshold_view(item));
    }
    views
}

pub fn sum_egress_threshold_view(view: &EgressThresholdView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_maximum + (parsed_sequence as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_threshold("code-325", "title-294");
        let view = to_egress_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
