use super::*;

use acme_kit::text;

/// How screens and exports show a query record.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRecordView {
    pub code_text: String,
    pub label_text: String,
    pub title: String,
    pub namespace_text: String,
    pub discount_text: String,
    pub key_id: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_query_record_view(item: &QueryRecord) -> QueryRecordView {
    QueryRecordView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        title: text::truncate(&item.title, 40),
        namespace_text: item.namespace.clone(),
        discount_text: format!("{:.2}", item.discount),
        key_id: item.key_id.clone(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_record_views(items: &[QueryRecord]) -> Vec<QueryRecordView> {
    let mut views: Vec<QueryRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_record_view(item));
    }
    views
}

pub fn sum_query_record_view(view: &QueryRecordView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_discount + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_record("code-182", "label-736");
        let view = to_query_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
