use super::*;

/// How screens and exports show a default query review.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewView {
    pub code: String,
    pub label: String,
    pub version: String,
    pub summary: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_default_query_review_view(item: &DefaultQueryReview) -> DefaultQueryReviewView {
    DefaultQueryReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        version: item.version.to_string(),
        summary: item.summary.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_query_review_views(items: &[DefaultQueryReview]) -> Vec<DefaultQueryReviewView> {
    let mut views: Vec<DefaultQueryReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_query_review_view(item));
    }
    views
}

pub fn sum_default_query_review_view(view: &DefaultQueryReviewView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_version as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_query_review("code-805", "label-10");
        let view = to_default_query_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_query_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
