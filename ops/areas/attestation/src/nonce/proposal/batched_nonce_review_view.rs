use super::*;

/// How screens and exports show a batched nonce review.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceReviewView {
    pub code_text: String,
    pub name_text: String,
    pub margin: String,
    pub archived: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_batched_nonce_review_view(item: &BatchedNonceReview) -> BatchedNonceReviewView {
    BatchedNonceReviewView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        archived: format!("{}", item.archived),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_nonce_review_views(items: &[BatchedNonceReview]) -> Vec<BatchedNonceReviewView> {
    let mut views: Vec<BatchedNonceReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_nonce_review_view(item));
    }
    views
}

pub fn sum_batched_nonce_review_view(view: &BatchedNonceReviewView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_margin + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_nonce_review("code-337", "name-847");
        let view = to_batched_nonce_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_nonce_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
