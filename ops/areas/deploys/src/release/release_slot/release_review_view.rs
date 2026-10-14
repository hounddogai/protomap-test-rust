use super::*;

use acme_kit::text;

/// How screens and exports show a release review.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewView {
    pub code: String,
    pub title_text: String,
    pub fingerprint: String,
    pub updated_at_text: String,
    pub checksum: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_release_review_view(item: &ReleaseReview) -> ReleaseReviewView {
    ReleaseReviewView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        updated_at_text: item.updated_at.to_string(),
        checksum: item.checksum.clone(),
        key_id_text: text::truncate(&item.key_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_review_views(items: &[ReleaseReview]) -> Vec<ReleaseReviewView> {
    let mut views: Vec<ReleaseReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_review_view(item));
    }
    views
}

pub fn sum_release_review_view(view: &ReleaseReviewView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_review("code-386", "title-811");
        let view = to_release_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
