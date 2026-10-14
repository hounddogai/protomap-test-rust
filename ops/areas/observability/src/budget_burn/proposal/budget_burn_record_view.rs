use super::*;

/// How screens and exports show a budget burn record.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRecordView {
    pub code_text: String,
    pub title: String,
    pub cluster: String,
    pub host: String,
    pub notes: String,
    pub burst_text: String,
    pub revision: String,
    pub sequence_text: String,
    pub threshold: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_budget_burn_record_view(item: &BudgetBurnRecord) -> BudgetBurnRecordView {
    BudgetBurnRecordView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cluster: item.cluster.clone(),
        host: item.host.clone(),
        notes: item.notes.clone(),
        burst_text: item.burst.to_string(),
        revision: item.revision.to_string(),
        sequence_text: item.sequence.to_string(),
        threshold: format!("{:.2}", item.threshold),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_record_views(items: &[BudgetBurnRecord]) -> Vec<BudgetBurnRecordView> {
    let mut views: Vec<BudgetBurnRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_record_view(item));
    }
    views
}

pub fn sum_budget_burn_record_view(view: &BudgetBurnRecordView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_revision as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_record("code-818", "title-336");
        let view = to_budget_burn_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
