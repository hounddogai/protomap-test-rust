use super::*;

use acme_kit::text;

/// How screens and exports show a page plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PagePlanView {
    pub code_text: String,
    pub label_text: String,
    pub notes: String,
    pub amount_text: String,
    pub quantity: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_page_plan_view(item: &PagePlan) -> PagePlanView {
    PagePlanView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        notes: item.notes.clone(),
        amount_text: format!("{:.2}", item.amount),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_plan_views(items: &[PagePlan]) -> Vec<PagePlanView> {
    let mut views: Vec<PagePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_plan_view(item));
    }
    views
}

pub fn sum_page_plan_view(view: &PagePlanView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_amount + (parsed_quantity as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_plan("code-23", "label-899");
        let view = to_page_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
