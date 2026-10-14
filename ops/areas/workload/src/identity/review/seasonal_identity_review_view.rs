use super::*;

/// How screens and exports show a seasonal identity review.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewView {
    pub code: String,
    pub title_text: String,
    pub ttl_seconds: String,
    pub segment_text: String,
    pub principal: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_seasonal_identity_review_view(item: &SeasonalIdentityReview) -> SeasonalIdentityReviewView {
    SeasonalIdentityReviewView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        segment_text: item.segment.clone(),
        principal: item.principal.clone(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_identity_review_views(items: &[SeasonalIdentityReview]) -> Vec<SeasonalIdentityReviewView> {
    let mut views: Vec<SeasonalIdentityReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_identity_review_view(item));
    }
    views
}

pub fn sum_seasonal_identity_review_view(view: &SeasonalIdentityReviewView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_identity_review("code-991", "title-295");
        let view = to_seasonal_identity_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_identity_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
