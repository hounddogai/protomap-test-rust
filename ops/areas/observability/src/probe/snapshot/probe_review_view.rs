use super::*;

use acme_kit::text;

/// How screens and exports show a probe review.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReviewView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub sequence_text: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_probe_review_view(item: &ProbeReview) -> ProbeReviewView {
    ProbeReviewView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 40),
        currency: item.currency.clone(),
        sequence_text: item.sequence.to_string(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_review_views(items: &[ProbeReview]) -> Vec<ProbeReviewView> {
    let mut views: Vec<ProbeReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_review_view(item));
    }
    views
}

pub fn sum_probe_review_view(view: &ProbeReviewView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_review("code-769", "label-585");
        let view = to_probe_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
