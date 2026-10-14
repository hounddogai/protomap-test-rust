use super::*;

use acme_kit::text;

/// How screens and exports show a pending silence estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceEstimateView {
    pub code: String,
    pub title_text: String,
    pub ends_at: String,
    pub namespace: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_pending_silence_estimate_view(item: &PendingSilenceEstimate) -> PendingSilenceEstimateView {
    PendingSilenceEstimateView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        namespace: item.namespace.clone(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_silence_estimate_views(items: &[PendingSilenceEstimate]) -> Vec<PendingSilenceEstimateView> {
    let mut views: Vec<PendingSilenceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_silence_estimate_view(item));
    }
    views
}

pub fn sum_pending_silence_estimate_view(view: &PendingSilenceEstimateView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_silence_estimate("code-424", "title-819");
        let view = to_pending_silence_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_silence_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
