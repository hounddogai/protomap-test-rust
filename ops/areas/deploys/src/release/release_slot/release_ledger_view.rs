use super::*;

use acme_kit::text;

/// How screens and exports show a release ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseLedgerView {
    pub code_text: String,
    pub title: String,
    pub category: String,
    pub region: String,
    pub cipher: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_release_ledger_view(item: &ReleaseLedger) -> ReleaseLedgerView {
    ReleaseLedgerView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        category: item.category.clone(),
        region: item.region.clone(),
        cipher: text::truncate(&item.cipher, 120),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_ledger_views(items: &[ReleaseLedger]) -> Vec<ReleaseLedgerView> {
    let mut views: Vec<ReleaseLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_ledger_view(item));
    }
    views
}

pub fn sum_release_ledger_view(view: &ReleaseLedgerView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_ledger("code-380", "title-611");
        let view = to_release_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
