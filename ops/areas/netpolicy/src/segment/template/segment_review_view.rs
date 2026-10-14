use super::*;

/// How screens and exports show a segment review.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReviewView {
    pub code_text: String,
    pub name_text: String,
    pub locale: String,
    pub burst: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_segment_review_view(item: &SegmentReview) -> SegmentReviewView {
    SegmentReviewView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        locale: item.locale.clone(),
        burst: item.burst.to_string(),
        title_text: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_review_views(items: &[SegmentReview]) -> Vec<SegmentReviewView> {
    let mut views: Vec<SegmentReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_review_view(item));
    }
    views
}

pub fn sum_segment_review_view(view: &SegmentReviewView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_review("code-924", "name-823");
        let view = to_segment_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
