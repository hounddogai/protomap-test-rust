use super::*;

/// How screens and exports show a silence review.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewView {
    pub code_text: String,
    pub title: String,
    pub summary: String,
    pub expires_at: String,
    pub quota_units: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_silence_review_view(item: &SilenceReview) -> SilenceReviewView {
    SilenceReviewView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        summary: item.summary.clone(),
        expires_at: item.expires_at.to_string(),
        quota_units: item.quota_units.to_string(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_review_views(items: &[SilenceReview]) -> Vec<SilenceReviewView> {
    let mut views: Vec<SilenceReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_review_view(item));
    }
    views
}

pub fn sum_silence_review_view(view: &SilenceReviewView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_quota_units as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_review("code-570", "title-959");
        let view = to_silence_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
