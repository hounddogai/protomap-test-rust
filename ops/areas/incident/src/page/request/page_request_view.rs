use super::*;

use acme_kit::text;

/// How screens and exports show a page request.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRequestView {
    pub code_text: String,
    pub label_text: String,
    pub starts_at: String,
    pub rotation_days: String,
    pub visible: String,
    pub sequence: String,
    pub region: String,
    pub retries_text: String,
    pub budget: String,
    pub window_days: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_page_request_view(item: &PageRequest) -> PageRequestView {
    PageRequestView {
        code_text: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        starts_at: item.starts_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        visible: format!("{}", item.visible),
        sequence: item.sequence.to_string(),
        region: item.region.clone(),
        retries_text: item.retries.to_string(),
        budget: format!("{:.2}", item.budget),
        window_days: item.window_days.to_string(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_request_views(items: &[PageRequest]) -> Vec<PageRequestView> {
    let mut views: Vec<PageRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_request_view(item));
    }
    views
}

pub fn sum_page_request_view(view: &PageRequestView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_rotation_days as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_request("code-347", "label-599");
        let view = to_page_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
