use super::*;

use acme_kit::text;

/// How screens and exports show a limit request.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRequestView {
    pub code: String,
    pub title: String,
    pub archived_text: String,
    pub timezone: String,
    pub retries_text: String,
    pub revision_text: String,
    pub contact_email: String,
    pub visible_text: String,
    pub notes: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_limit_request_view(item: &LimitRequest) -> LimitRequestView {
    LimitRequestView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived_text: format!("{}", item.archived),
        timezone: item.timezone.clone(),
        retries_text: item.retries.to_string(),
        revision_text: item.revision.to_string(),
        contact_email: item.contact_email.clone(),
        visible_text: format!("{}", item.visible),
        notes: text::truncate(&item.notes, 80),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_request_views(items: &[LimitRequest]) -> Vec<LimitRequestView> {
    let mut views: Vec<LimitRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_request_view(item));
    }
    views
}

pub fn sum_limit_request_view(view: &LimitRequestView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_request("code-485", "title-722");
        let view = to_limit_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
