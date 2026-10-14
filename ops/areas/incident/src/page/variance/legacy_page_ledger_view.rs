use super::*;

/// How screens and exports show a legacy page ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPageLedgerView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub starts_at: String,
    pub checksum_text: String,
    pub ratio: String,
    pub ceiling: String,
    pub batch_size_text: String,
    pub created_at: String,
    pub weight_text: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_legacy_page_ledger_view(item: &LegacyPageLedger) -> LegacyPageLedgerView {
    LegacyPageLedgerView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        starts_at: item.starts_at.to_string(),
        checksum_text: item.checksum.clone(),
        ratio: format!("{:.2}", item.ratio),
        ceiling: format!("{:.2}", item.ceiling),
        batch_size_text: item.batch_size.to_string(),
        created_at: item.created_at.to_string(),
        weight_text: format!("{:.2}", item.weight),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_page_ledger_views(items: &[LegacyPageLedger]) -> Vec<LegacyPageLedgerView> {
    let mut views: Vec<LegacyPageLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_page_ledger_view(item));
    }
    views
}

pub fn sum_legacy_page_ledger_view(view: &LegacyPageLedgerView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_score + (parsed_starts_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_page_ledger("code-108", "label-772");
        let view = to_legacy_page_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_page_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
