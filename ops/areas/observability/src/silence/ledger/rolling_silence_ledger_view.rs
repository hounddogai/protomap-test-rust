use super::*;

use acme_kit::text;

/// How screens and exports show a rolling silence ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceLedgerView {
    pub code: String,
    pub title: String,
    pub label: String,
    pub external_ref_text: String,
    pub cluster: String,
    pub ceiling: String,
    pub locale: String,
    pub key_id: String,
    pub namespace_text: String,
    pub region: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_rolling_silence_ledger_view(item: &RollingSilenceLedger) -> RollingSilenceLedgerView {
    RollingSilenceLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        label: item.label.clone(),
        external_ref_text: item.external_ref.clone(),
        cluster: text::truncate(&item.cluster, 64),
        ceiling: format!("{:.2}", item.ceiling),
        locale: item.locale.clone(),
        key_id: item.key_id.clone(),
        namespace_text: text::truncate(&item.namespace, 64),
        region: text::truncate(&item.region, 80),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_silence_ledger_views(items: &[RollingSilenceLedger]) -> Vec<RollingSilenceLedgerView> {
    let mut views: Vec<RollingSilenceLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_silence_ledger_view(item));
    }
    views
}

pub fn sum_rolling_silence_ledger_view(view: &RollingSilenceLedgerView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ceiling + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_silence_ledger("code-262", "title-677");
        let view = to_rolling_silence_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_silence_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
