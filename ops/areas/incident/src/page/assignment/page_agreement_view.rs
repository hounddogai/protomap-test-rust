use super::*;

/// How screens and exports show a page agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAgreementView {
    pub code_text: String,
    pub label: String,
    pub reviewer_text: String,
    pub port: String,
    pub burst: String,
    pub active_text: String,
    pub title_text: String,
    pub starts_at: String,
    pub approved: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_page_agreement_view(item: &PageAgreement) -> PageAgreementView {
    PageAgreementView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        reviewer_text: item.reviewer.clone(),
        port: item.port.to_string(),
        burst: item.burst.to_string(),
        active_text: format!("{}", item.active),
        title_text: item.title.clone(),
        starts_at: item.starts_at.to_string(),
        approved: format!("{}", item.approved),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_agreement_views(items: &[PageAgreement]) -> Vec<PageAgreementView> {
    let mut views: Vec<PageAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_agreement_view(item));
    }
    views
}

pub fn sum_page_agreement_view(view: &PageAgreementView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_burst as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_agreement("code-53", "label-388");
        let view = to_page_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
