use super::*;

/// How screens and exports show a batched runbook summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSummaryView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub approved: String,
    pub key_id: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_batched_runbook_summary_view(item: &BatchedRunbookSummary) -> BatchedRunbookSummaryView {
    BatchedRunbookSummaryView {
        code: item.code.clone(),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        approved: format!("{}", item.approved),
        key_id: item.key_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_runbook_summary_views(items: &[BatchedRunbookSummary]) -> Vec<BatchedRunbookSummaryView> {
    let mut views: Vec<BatchedRunbookSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_runbook_summary_view(item));
    }
    views
}

pub fn sum_batched_runbook_summary_view(view: &BatchedRunbookSummaryView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ceiling + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_runbook_summary("code-723", "name-908");
        let view = to_batched_runbook_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_runbook_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
