use super::*;

use acme_kit::text;

/// How screens and exports show a default action item exception.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemExceptionView {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub ends_at_text: String,
    pub batch_size: String,
    pub external_ref_text: String,
    pub category_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_default_action_item_exception_view(item: &DefaultActionItemException) -> DefaultActionItemExceptionView {
    DefaultActionItemExceptionView {
        code: item.code.clone(),
        name: item.name.clone(),
        namespace: text::truncate(&item.namespace, 80),
        ends_at_text: item.ends_at.to_string(),
        batch_size: item.batch_size.to_string(),
        external_ref_text: text::truncate(&item.external_ref, 64),
        category_text: item.category.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_action_item_exception_views(
    items: &[DefaultActionItemException],
) -> Vec<DefaultActionItemExceptionView> {
    let mut views: Vec<DefaultActionItemExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_action_item_exception_view(item));
    }
    views
}

pub fn sum_default_action_item_exception_view(view: &DefaultActionItemExceptionView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_action_item_exception("code-623", "name-883");
        let view = to_default_action_item_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_action_item_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
