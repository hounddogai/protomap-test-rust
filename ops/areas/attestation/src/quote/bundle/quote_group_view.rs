use super::*;

use acme_kit::text;

/// How screens and exports show a quote group.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroupView {
    pub code: String,
    pub name_text: String,
    pub principal: String,
    pub contact_email: String,
    pub grace_days_text: String,
    pub weight: String,
    pub tenant_id: String,
    pub active: String,
    pub namespace: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_quote_group_view(item: &QuoteGroup) -> QuoteGroupView {
    QuoteGroupView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        principal: item.principal.clone(),
        contact_email: item.contact_email.clone(),
        grace_days_text: item.grace_days.to_string(),
        weight: format!("{:.2}", item.weight),
        tenant_id: text::truncate(&item.tenant_id, 40),
        active: format!("{}", item.active),
        namespace: item.namespace.clone(),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_group_views(items: &[QuoteGroup]) -> Vec<QuoteGroupView> {
    let mut views: Vec<QuoteGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_group_view(item));
    }
    views
}

pub fn sum_quote_group_view(view: &QuoteGroupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_group("code-952", "name-227");
        let view = to_quote_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
