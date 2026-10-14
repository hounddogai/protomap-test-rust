use super::*;

/// How screens and exports show a pending overage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingOverageBatchView {
    pub code_text: String,
    pub label: String,
    pub tenant_id: String,
    pub threshold_text: String,
    pub rotation_days: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_pending_overage_batch_view(item: &PendingOverageBatch) -> PendingOverageBatchView {
    PendingOverageBatchView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        tenant_id: item.tenant_id.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        rotation_days: item.rotation_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_overage_batch_views(items: &[PendingOverageBatch]) -> Vec<PendingOverageBatchView> {
    let mut views: Vec<PendingOverageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_overage_batch_view(item));
    }
    views
}

pub fn sum_pending_overage_batch_view(view: &PendingOverageBatchView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_threshold + (parsed_rotation_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_overage_batch("code-220", "label-308");
        let view = to_pending_overage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_overage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
