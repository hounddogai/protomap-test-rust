use super::*;

use acme_kit::text;

/// How screens and exports show a rollback request.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestView {
    pub code: String,
    pub label_text: String,
    pub audience_text: String,
    pub discount: String,
    pub sequence: String,
    pub ceiling: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_rollback_request_view(item: &RollbackRequest) -> RollbackRequestView {
    RollbackRequestView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        audience_text: item.audience.clone(),
        discount: format!("{:.2}", item.discount),
        sequence: item.sequence.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        cipher: text::truncate(&item.cipher, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_request_views(items: &[RollbackRequest]) -> Vec<RollbackRequestView> {
    let mut views: Vec<RollbackRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_request_view(item));
    }
    views
}

pub fn sum_rollback_request_view(view: &RollbackRequestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_discount + (parsed_sequence as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_request("code-759", "label-924");
        let view = to_rollback_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
