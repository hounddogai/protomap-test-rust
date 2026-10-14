use super::*;

use acme_kit::text;

/// How screens and exports show a rollback batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatchView {
    pub code: String,
    pub title: String,
    pub category_text: String,
    pub label_text: String,
    pub owner_id: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_rollback_batch_view(item: &RollbackBatch) -> RollbackBatchView {
    RollbackBatchView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        category_text: item.category.clone(),
        label_text: text::truncate(&item.label, 80),
        owner_id: item.owner_id.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_batch_views(items: &[RollbackBatch]) -> Vec<RollbackBatchView> {
    let mut views: Vec<RollbackBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_batch_view(item));
    }
    views
}

pub fn sum_rollback_batch_view(view: &RollbackBatchView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_batch("code-208", "title-806");
        let view = to_rollback_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
