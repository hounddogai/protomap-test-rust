use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn batch.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchView {
    pub code_text: String,
    pub label: String,
    pub unit: String,
    pub contact_email: String,
    pub segment: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_budget_burn_batch_view(item: &BudgetBurnBatch) -> BudgetBurnBatchView {
    BudgetBurnBatchView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        unit: item.unit.clone(),
        contact_email: item.contact_email.clone(),
        segment: text::truncate(&item.segment, 40),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_batch_views(items: &[BudgetBurnBatch]) -> Vec<BudgetBurnBatchView> {
    let mut views: Vec<BudgetBurnBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_batch_view(item));
    }
    views
}

pub fn sum_budget_burn_batch_view(view: &BudgetBurnBatchView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_batch("code-146", "label-582");
        let view = to_budget_burn_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
