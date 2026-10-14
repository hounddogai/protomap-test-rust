use super::*;

/// How screens and exports show a nonce notice.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceNoticeView {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub name_text: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_nonce_notice_view(item: &NonceNotice) -> NonceNoticeView {
    NonceNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        reviewer: item.reviewer.clone(),
        name_text: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_notice_views(items: &[NonceNotice]) -> Vec<NonceNoticeView> {
    let mut views: Vec<NonceNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_notice_view(item));
    }
    views
}

pub fn sum_nonce_notice_view(view: &NonceNoticeView) -> Result<f64, String> {
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
        let item = new_nonce_notice("code-616", "label-774");
        let view = to_nonce_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
