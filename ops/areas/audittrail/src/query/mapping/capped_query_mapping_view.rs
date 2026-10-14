use super::*;

use acme_kit::text;

/// How screens and exports show a capped query mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryMappingView {
    pub code: String,
    pub name_text: String,
    pub contact_email: String,
    pub locale_text: String,
    pub principal_text: String,
    pub reviewer: String,
    pub active: String,
    pub visible: String,
    pub category: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_capped_query_mapping_view(item: &CappedQueryMapping) -> CappedQueryMappingView {
    CappedQueryMappingView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 40),
        contact_email: text::truncate(&item.contact_email, 120),
        locale_text: text::truncate(&item.locale, 120),
        principal_text: item.principal.clone(),
        reviewer: item.reviewer.clone(),
        active: format!("{}", item.active),
        visible: format!("{}", item.visible),
        category: item.category.clone(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_query_mapping_views(items: &[CappedQueryMapping]) -> Vec<CappedQueryMappingView> {
    let mut views: Vec<CappedQueryMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_query_mapping_view(item));
    }
    views
}

pub fn sum_capped_query_mapping_view(view: &CappedQueryMappingView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_query_mapping("code-963", "name-324");
        let view = to_capped_query_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_query_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
