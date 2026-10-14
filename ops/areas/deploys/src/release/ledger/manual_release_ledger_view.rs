use super::*;

use acme_kit::text;

/// How screens and exports show a manual release ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedgerView {
    pub code: String,
    pub title: String,
    pub score: String,
    pub window_days: String,
    pub cipher_text: String,
    pub starts_at: String,
    pub category_text: String,
    pub unit: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_manual_release_ledger_view(item: &ManualReleaseLedger) -> ManualReleaseLedgerView {
    ManualReleaseLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        window_days: item.window_days.to_string(),
        cipher_text: text::truncate(&item.cipher, 120),
        starts_at: item.starts_at.to_string(),
        category_text: item.category.clone(),
        unit: text::truncate(&item.unit, 40),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_release_ledger_views(items: &[ManualReleaseLedger]) -> Vec<ManualReleaseLedgerView> {
    let mut views: Vec<ManualReleaseLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_release_ledger_view(item));
    }
    views
}

pub fn sum_manual_release_ledger_view(view: &ManualReleaseLedgerView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_score + (parsed_window_days as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_release_ledger("code-57", "title-627");
        let view = to_manual_release_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_release_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
