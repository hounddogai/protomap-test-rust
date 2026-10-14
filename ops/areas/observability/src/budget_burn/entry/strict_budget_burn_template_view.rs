use super::*;

use acme_kit::text;

/// How screens and exports show a strict budget burn template.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateView {
    pub code: String,
    pub label_text: String,
    pub checksum: String,
    pub locale_text: String,
    pub tenant_id: String,
    pub created_at: String,
    pub country: String,
    pub phone_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_strict_budget_burn_template_view(item: &StrictBudgetBurnTemplate) -> StrictBudgetBurnTemplateView {
    StrictBudgetBurnTemplateView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        checksum: text::truncate(&item.checksum, 80),
        locale_text: item.locale.clone(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        created_at: item.created_at.to_string(),
        country: text::truncate(&item.country, 80),
        phone_text: item.phone.clone(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_budget_burn_template_views(items: &[StrictBudgetBurnTemplate]) -> Vec<StrictBudgetBurnTemplateView> {
    let mut views: Vec<StrictBudgetBurnTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_budget_burn_template_view(item));
    }
    views
}

pub fn sum_strict_budget_burn_template_view(view: &StrictBudgetBurnTemplateView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_budget_burn_template("code-977", "label-929");
        let view = to_strict_budget_burn_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_budget_burn_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
