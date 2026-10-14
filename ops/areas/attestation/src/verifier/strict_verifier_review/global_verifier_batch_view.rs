use super::*;

use acme_kit::text;

/// How screens and exports show a global verifier batch.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierBatchView {
    pub code: String,
    pub label_text: String,
    pub minimum: String,
    pub ceiling: String,
    pub host_text: String,
    pub title: String,
    pub score: String,
    pub discount_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_global_verifier_batch_view(item: &GlobalVerifierBatch) -> GlobalVerifierBatchView {
    GlobalVerifierBatchView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        minimum: format!("{:.2}", item.minimum),
        ceiling: format!("{:.2}", item.ceiling),
        host_text: item.host.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        discount_text: format!("{:.2}", item.discount),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_verifier_batch_views(items: &[GlobalVerifierBatch]) -> Vec<GlobalVerifierBatchView> {
    let mut views: Vec<GlobalVerifierBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_verifier_batch_view(item));
    }
    views
}

pub fn sum_global_verifier_batch_view(view: &GlobalVerifierBatchView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_minimum + parsed_ceiling + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_verifier_batch("code-839", "label-231");
        let view = to_global_verifier_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_verifier_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
