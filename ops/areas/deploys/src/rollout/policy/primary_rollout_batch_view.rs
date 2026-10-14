use super::*;

/// How screens and exports show a primary rollout batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatchView {
    pub code: String,
    pub name_text: String,
    pub owner_id: String,
    pub port: String,
    pub rate: String,
    pub sequence_text: String,
    pub unit: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_primary_rollout_batch_view(item: &PrimaryRolloutBatch) -> PrimaryRolloutBatchView {
    PrimaryRolloutBatchView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        owner_id: item.owner_id.clone(),
        port: item.port.to_string(),
        rate: format!("{:.2}", item.rate),
        sequence_text: item.sequence.to_string(),
        unit: item.unit.clone(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rollout_batch_views(items: &[PrimaryRolloutBatch]) -> Vec<PrimaryRolloutBatchView> {
    let mut views: Vec<PrimaryRolloutBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rollout_batch_view(item));
    }
    views
}

pub fn sum_primary_rollout_batch_view(view: &PrimaryRolloutBatchView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_port as f64 + parsed_rate + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rollout_batch("code-754", "name-800");
        let view = to_primary_rollout_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rollout_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
