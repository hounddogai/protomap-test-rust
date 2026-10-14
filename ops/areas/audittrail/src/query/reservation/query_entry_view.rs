use super::*;

use acme_kit::text;

/// How screens and exports show a query entry.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryView {
    pub code: String,
    pub name: String,
    pub visible: String,
    pub unit: String,
    pub external_ref_text: String,
    pub principal_text: String,
    pub issuer: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_query_entry_view(item: &QueryEntry) -> QueryEntryView {
    QueryEntryView {
        code: item.code.clone(),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        unit: text::truncate(&item.unit, 120),
        external_ref_text: item.external_ref.clone(),
        principal_text: item.principal.clone(),
        issuer: item.issuer.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_entry_views(items: &[QueryEntry]) -> Vec<QueryEntryView> {
    let mut views: Vec<QueryEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_entry_view(item));
    }
    views
}

pub fn sum_query_entry_view(view: &QueryEntryView) -> Result<f64, String> {
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
        let item = new_query_entry("code-415", "name-290");
        let view = to_query_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
