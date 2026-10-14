use super::*;

use acme_kit::text;

/// How screens and exports show a quote notice.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteNoticeView {
    pub code: String,
    pub name_text: String,
    pub pinned: String,
    pub ceiling: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_quote_notice_view(item: &QuoteNotice) -> QuoteNoticeView {
    QuoteNoticeView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        pinned: format!("{}", item.pinned),
        ceiling: format!("{:.2}", item.ceiling),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_notice_views(items: &[QuoteNotice]) -> Vec<QuoteNoticeView> {
    let mut views: Vec<QuoteNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_notice_view(item));
    }
    views
}

pub fn sum_quote_notice_view(view: &QuoteNoticeView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_quote_notice("code-481", "name-271");
        let view = to_quote_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_quote_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
