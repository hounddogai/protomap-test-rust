use super::*;

use acme_kit::text;

/// How screens and exports show a weighted overage audit.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageAuditView {
    pub code: String,
    pub title_text: String,
    pub audience_text: String,
    pub principal: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_weighted_overage_audit_view(item: &WeightedOverageAudit) -> WeightedOverageAuditView {
    WeightedOverageAuditView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        audience_text: text::truncate(&item.audience, 64),
        principal: text::truncate(&item.principal, 40),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_overage_audit_views(items: &[WeightedOverageAudit]) -> Vec<WeightedOverageAuditView> {
    let mut views: Vec<WeightedOverageAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_overage_audit_view(item));
    }
    views
}

pub fn sum_weighted_overage_audit_view(view: &WeightedOverageAuditView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
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
        let item = new_weighted_overage_audit("code-160", "title-887");
        let view = to_weighted_overage_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_overage_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
