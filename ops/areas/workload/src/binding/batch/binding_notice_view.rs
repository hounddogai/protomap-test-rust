use super::*;

use acme_kit::text;

/// How screens and exports show a binding notice.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingNoticeView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub burst: String,
    pub margin: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_binding_notice_view(item: &BindingNotice) -> BindingNoticeView {
    BindingNoticeView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        burst: item.burst.to_string(),
        margin: format!("{:.2}", item.margin),
        notes_text: text::truncate(&item.notes, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_notice_views(items: &[BindingNotice]) -> Vec<BindingNoticeView> {
    let mut views: Vec<BindingNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_notice_view(item));
    }
    views
}

pub fn sum_binding_notice_view(view: &BindingNoticeView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_amount + (parsed_burst as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_notice("code-996", "name-475");
        let view = to_binding_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
