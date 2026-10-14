use super::*;

use acme_kit::text;

/// How screens and exports show a lenient action item batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchView {
    pub code: String,
    pub label_text: String,
    pub revision_text: String,
    pub cipher_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_lenient_action_item_batch_view(item: &LenientActionItemBatch) -> LenientActionItemBatchView {
    LenientActionItemBatchView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        revision_text: item.revision.to_string(),
        cipher_text: text::truncate(&item.cipher, 64),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_action_item_batch_views(items: &[LenientActionItemBatch]) -> Vec<LenientActionItemBatchView> {
    let mut views: Vec<LenientActionItemBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_action_item_batch_view(item));
    }
    views
}

pub fn sum_lenient_action_item_batch_view(view: &LenientActionItemBatchView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_action_item_batch("code-352", "label-728");
        let view = to_lenient_action_item_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_action_item_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
