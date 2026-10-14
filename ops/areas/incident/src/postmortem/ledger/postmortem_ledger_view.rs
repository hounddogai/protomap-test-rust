use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedgerView {
    pub code: String,
    pub title: String,
    pub active_text: String,
    pub retries: String,
    pub approved: String,
    pub pinned: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_postmortem_ledger_view(item: &PostmortemLedger) -> PostmortemLedgerView {
    PostmortemLedgerView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        active_text: format!("{}", item.active),
        retries: item.retries.to_string(),
        approved: format!("{}", item.approved),
        pinned: format!("{}", item.pinned),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_ledger_views(items: &[PostmortemLedger]) -> Vec<PostmortemLedgerView> {
    let mut views: Vec<PostmortemLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_ledger_view(item));
    }
    views
}

pub fn sum_postmortem_ledger_view(view: &PostmortemLedgerView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_ledger("code-119", "title-345");
        let view = to_postmortem_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
