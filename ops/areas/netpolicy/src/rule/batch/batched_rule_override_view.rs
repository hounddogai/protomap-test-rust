use super::*;

use acme_kit::text;

/// How screens and exports show a batched rule override.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleOverrideView {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub checksum: String,
    pub contact_email_text: String,
    pub phone: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_batched_rule_override_view(item: &BatchedRuleOverride) -> BatchedRuleOverrideView {
    BatchedRuleOverrideView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        checksum: item.checksum.clone(),
        contact_email_text: text::truncate(&item.contact_email, 64),
        phone: text::truncate(&item.phone, 40),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_rule_override_views(items: &[BatchedRuleOverride]) -> Vec<BatchedRuleOverrideView> {
    let mut views: Vec<BatchedRuleOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_rule_override_view(item));
    }
    views
}

pub fn sum_batched_rule_override_view(view: &BatchedRuleOverrideView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_rule_override("code-714", "title-619");
        let view = to_batched_rule_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_rule_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
