use super::*;

use acme_kit::text;

/// How screens and exports show a rule ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedgerView {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub discount: String,
    pub source_text: String,
    pub checksum_text: String,
    pub threshold: String,
    pub visible: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_rule_ledger_view(item: &RuleLedger) -> RuleLedgerView {
    RuleLedgerView {
        code: item.code.clone(),
        title: item.title.clone(),
        external_ref: text::truncate(&item.external_ref, 120),
        discount: format!("{:.2}", item.discount),
        source_text: item.source.clone(),
        checksum_text: item.checksum.clone(),
        threshold: format!("{:.2}", item.threshold),
        visible: format!("{}", item.visible),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_ledger_views(items: &[RuleLedger]) -> Vec<RuleLedgerView> {
    let mut views: Vec<RuleLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_ledger_view(item));
    }
    views
}

pub fn sum_rule_ledger_view(view: &RuleLedgerView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_discount + parsed_threshold + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_ledger("code-549", "title-328");
        let view = to_rule_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
