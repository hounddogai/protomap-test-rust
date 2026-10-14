use super::*;

use acme_kit::text;

/// How screens and exports show a slo review.
#[derive(Debug, Clone, PartialEq)]
pub struct SloReviewView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub checksum_text: String,
    pub weight: String,
    pub source_text: String,
    pub label: String,
    pub discount: String,
    pub sequence_text: String,
    pub host: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_slo_review_view(item: &SloReview) -> SloReviewView {
    SloReviewView {
        code: item.code.clone(),
        name: item.name.clone(),
        country: item.country.clone(),
        checksum_text: text::truncate(&item.checksum, 64),
        weight: format!("{:.2}", item.weight),
        source_text: item.source.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        sequence_text: item.sequence.to_string(),
        host: item.host.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_review_views(items: &[SloReview]) -> Vec<SloReviewView> {
    let mut views: Vec<SloReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_review_view(item));
    }
    views
}

pub fn sum_slo_review_view(view: &SloReviewView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_weight + parsed_discount + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_review("code-583", "name-644");
        let view = to_slo_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
