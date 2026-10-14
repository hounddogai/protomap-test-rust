use super::*;

/// How screens and exports show a lenient zone review.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReviewView {
    pub code: String,
    pub label: String,
    pub burst: String,
    pub window_days: String,
    pub priority: String,
    pub threshold: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_lenient_zone_review_view(item: &LenientZoneReview) -> LenientZoneReviewView {
    LenientZoneReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        window_days: item.window_days.to_string(),
        priority: item.priority.to_string(),
        threshold: format!("{:.2}", item.threshold),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_zone_review_views(items: &[LenientZoneReview]) -> Vec<LenientZoneReviewView> {
    let mut views: Vec<LenientZoneReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_zone_review_view(item));
    }
    views
}

pub fn sum_lenient_zone_review_view(view: &LenientZoneReviewView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_window_days as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_zone_review("code-438", "label-915");
        let view = to_lenient_zone_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_zone_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
