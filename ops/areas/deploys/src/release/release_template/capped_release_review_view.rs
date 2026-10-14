use super::*;

/// How screens and exports show a capped release review.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseReviewView {
    pub code_text: String,
    pub name_text: String,
    pub host: String,
    pub quota_units: String,
    pub scope: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_capped_release_review_view(item: &CappedReleaseReview) -> CappedReleaseReviewView {
    CappedReleaseReviewView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        host: item.host.clone(),
        quota_units: item.quota_units.to_string(),
        scope: item.scope.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_release_review_views(items: &[CappedReleaseReview]) -> Vec<CappedReleaseReviewView> {
    let mut views: Vec<CappedReleaseReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_release_review_view(item));
    }
    views
}

pub fn sum_capped_release_review_view(view: &CappedReleaseReviewView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_release_review("code-589", "name-211");
        let view = to_capped_release_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_release_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
