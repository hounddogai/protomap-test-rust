use super::*;

use acme_kit::text;

/// How screens and exports show a batched page threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThresholdView {
    pub code_text: String,
    pub name_text: String,
    pub expires_at_text: String,
    pub discount: String,
    pub pinned: String,
    pub ceiling: String,
    pub category_text: String,
    pub namespace: String,
    pub reviewer_text: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_batched_page_threshold_view(item: &BatchedPageThreshold) -> BatchedPageThresholdView {
    BatchedPageThresholdView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        expires_at_text: item.expires_at.to_string(),
        discount: format!("{:.2}", item.discount),
        pinned: format!("{}", item.pinned),
        ceiling: format!("{:.2}", item.ceiling),
        category_text: text::truncate(&item.category, 40),
        namespace: text::truncate(&item.namespace, 40),
        reviewer_text: text::truncate(&item.reviewer, 80),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_page_threshold_views(items: &[BatchedPageThreshold]) -> Vec<BatchedPageThresholdView> {
    let mut views: Vec<BatchedPageThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_page_threshold_view(item));
    }
    views
}

pub fn sum_batched_page_threshold_view(view: &BatchedPageThresholdView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_discount + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_page_threshold("code-437", "name-46");
        let view = to_batched_page_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_page_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
