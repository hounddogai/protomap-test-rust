use super::*;

use acme_kit::text;

/// How screens and exports show a isolation request.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRequestView {
    pub code: String,
    pub label_text: String,
    pub cipher: String,
    pub title: String,
    pub notes: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_isolation_request_view(item: &IsolationRequest) -> IsolationRequestView {
    IsolationRequestView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        cipher: item.cipher.clone(),
        title: item.title.clone(),
        notes: item.notes.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_request_views(items: &[IsolationRequest]) -> Vec<IsolationRequestView> {
    let mut views: Vec<IsolationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_request_view(item));
    }
    views
}

pub fn sum_isolation_request_view(view: &IsolationRequestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_request("code-751", "label-655");
        let view = to_isolation_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
