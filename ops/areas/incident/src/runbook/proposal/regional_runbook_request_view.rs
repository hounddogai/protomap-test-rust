use super::*;

use acme_kit::text;

/// How screens and exports show a regional runbook request.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookRequestView {
    pub code: String,
    pub label_text: String,
    pub batch_size: String,
    pub ends_at_text: String,
    pub score: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_regional_runbook_request_view(item: &RegionalRunbookRequest) -> RegionalRunbookRequestView {
    RegionalRunbookRequestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        ends_at_text: item.ends_at.to_string(),
        score: format!("{:.2}", item.score),
        contact_email: text::truncate(&item.contact_email, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_runbook_request_views(items: &[RegionalRunbookRequest]) -> Vec<RegionalRunbookRequestView> {
    let mut views: Vec<RegionalRunbookRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_runbook_request_view(item));
    }
    views
}

pub fn sum_regional_runbook_request_view(view: &RegionalRunbookRequestView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_ends_at as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_runbook_request("code-313", "label-434");
        let view = to_regional_runbook_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_runbook_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
