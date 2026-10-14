use super::*;

use acme_kit::text;

/// How screens and exports show a slo batch.
#[derive(Debug, Clone, PartialEq)]
pub struct SloBatchView {
    pub code_text: String,
    pub title: String,
    pub threshold: String,
    pub summary: String,
    pub weight: String,
    pub label: String,
    pub pinned: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_slo_batch_view(item: &SloBatch) -> SloBatchView {
    SloBatchView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 64),
        threshold: format!("{:.2}", item.threshold),
        summary: item.summary.clone(),
        weight: format!("{:.2}", item.weight),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_batch_views(items: &[SloBatch]) -> Vec<SloBatchView> {
    let mut views: Vec<SloBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_batch_view(item));
    }
    views
}

pub fn sum_slo_batch_view(view: &SloBatchView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_threshold + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_batch("code-85", "title-608");
        let view = to_slo_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
