use super::*;

use acme_kit::text;

/// How screens and exports show a fallback page ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedgerView {
    pub code_text: String,
    pub name: String,
    pub country_text: String,
    pub rotation_days_text: String,
    pub cluster_text: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_fallback_page_ledger_view(item: &FallbackPageLedger) -> FallbackPageLedgerView {
    FallbackPageLedgerView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        country_text: text::truncate(&item.country, 80),
        rotation_days_text: item.rotation_days.to_string(),
        cluster_text: item.cluster.clone(),
        key_id_text: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_page_ledger_views(items: &[FallbackPageLedger]) -> Vec<FallbackPageLedgerView> {
    let mut views: Vec<FallbackPageLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_page_ledger_view(item));
    }
    views
}

pub fn sum_fallback_page_ledger_view(view: &FallbackPageLedgerView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_page_ledger("code-316", "name-23");
        let view = to_fallback_page_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_page_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
