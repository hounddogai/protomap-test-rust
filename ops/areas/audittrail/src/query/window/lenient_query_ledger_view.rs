use super::*;

use acme_kit::text;

/// How screens and exports show a lenient query ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedgerView {
    pub code: String,
    pub label: String,
    pub retries_text: String,
    pub ratio: String,
    pub rate: String,
    pub revision: String,
    pub channel_text: String,
    pub segment: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_lenient_query_ledger_view(item: &LenientQueryLedger) -> LenientQueryLedgerView {
    LenientQueryLedgerView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        retries_text: item.retries.to_string(),
        ratio: format!("{:.2}", item.ratio),
        rate: format!("{:.2}", item.rate),
        revision: item.revision.to_string(),
        channel_text: item.channel.clone(),
        segment: item.segment.clone(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_query_ledger_views(items: &[LenientQueryLedger]) -> Vec<LenientQueryLedgerView> {
    let mut views: Vec<LenientQueryLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_query_ledger_view(item));
    }
    views
}

pub fn sum_lenient_query_ledger_view(view: &LenientQueryLedgerView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_query_ledger("code-19", "label-767");
        let view = to_lenient_query_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_query_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
