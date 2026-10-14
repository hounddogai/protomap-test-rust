use super::*;

/// How screens and exports show a rollback score.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScoreView {
    pub code: String,
    pub label: String,
    pub burst: String,
    pub capacity: String,
    pub ceiling: String,
    pub tenant_id: String,
    pub sequence_text: String,
    pub maximum_text: String,
    pub priority_text: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_rollback_score_view(item: &RollbackScore) -> RollbackScoreView {
    RollbackScoreView {
        code: item.code.clone(),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        capacity: item.capacity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        tenant_id: item.tenant_id.clone(),
        sequence_text: item.sequence.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        priority_text: item.priority.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_score_views(items: &[RollbackScore]) -> Vec<RollbackScoreView> {
    let mut views: Vec<RollbackScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_score_view(item));
    }
    views
}

pub fn sum_rollback_score_view(view: &RollbackScoreView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_capacity as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_score("code-309", "label-622");
        let view = to_rollback_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
