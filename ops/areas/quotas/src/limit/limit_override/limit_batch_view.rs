use super::*;

use acme_kit::text;

/// How screens and exports show a limit batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitBatchView {
    pub code: String,
    pub title_text: String,
    pub updated_at: String,
    pub threshold_text: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_limit_batch_view(item: &LimitBatch) -> LimitBatchView {
    LimitBatchView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        country: text::truncate(&item.country, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_batch_views(items: &[LimitBatch]) -> Vec<LimitBatchView> {
    let mut views: Vec<LimitBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_batch_view(item));
    }
    views
}

pub fn sum_limit_batch_view(view: &LimitBatchView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_batch("code-399", "title-365");
        let view = to_limit_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
