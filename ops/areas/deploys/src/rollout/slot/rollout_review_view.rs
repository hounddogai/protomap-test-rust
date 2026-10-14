use super::*;

use acme_kit::text;

/// How screens and exports show a rollout review.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReviewView {
    pub code: String,
    pub title_text: String,
    pub score: String,
    pub margin: String,
    pub cipher_text: String,
    pub host_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_rollout_review_view(item: &RolloutReview) -> RolloutReviewView {
    RolloutReviewView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        score: format!("{:.2}", item.score),
        margin: format!("{:.2}", item.margin),
        cipher_text: item.cipher.clone(),
        host_text: text::truncate(&item.host, 120),
        region: text::truncate(&item.region, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_review_views(items: &[RolloutReview]) -> Vec<RolloutReviewView> {
    let mut views: Vec<RolloutReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_review_view(item));
    }
    views
}

pub fn sum_rollout_review_view(view: &RolloutReviewView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_score + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_review("code-682", "title-536");
        let view = to_rollout_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
