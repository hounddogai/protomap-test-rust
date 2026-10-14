use super::*;

/// How screens and exports show a escalation checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationCheckpointView {
    pub code: String,
    pub label_text: String,
    pub threshold: String,
    pub batch_size_text: String,
    pub ttl_seconds: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_escalation_checkpoint_view(item: &EscalationCheckpoint) -> EscalationCheckpointView {
    EscalationCheckpointView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        threshold: format!("{:.2}", item.threshold),
        batch_size_text: item.batch_size.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_checkpoint_views(items: &[EscalationCheckpoint]) -> Vec<EscalationCheckpointView> {
    let mut views: Vec<EscalationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_checkpoint_view(item));
    }
    views
}

pub fn sum_escalation_checkpoint_view(view: &EscalationCheckpointView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_threshold + (parsed_batch_size as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_checkpoint("code-739", "label-665");
        let view = to_escalation_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
