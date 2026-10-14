use super::*;

use acme_kit::text;

/// How screens and exports show a silence ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceLedgerView {
    pub code_text: String,
    pub name: String,
    pub cost: String,
    pub summary: String,
    pub burst: String,
    pub category: String,
    pub capacity: String,
    pub archived: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_silence_ledger_view(item: &SilenceLedger) -> SilenceLedgerView {
    SilenceLedgerView {
        code_text: text::truncate(&item.code, 40),
        name: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        summary: item.summary.clone(),
        burst: item.burst.to_string(),
        category: item.category.clone(),
        capacity: item.capacity.to_string(),
        archived: format!("{}", item.archived),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_ledger_views(items: &[SilenceLedger]) -> Vec<SilenceLedgerView> {
    let mut views: Vec<SilenceLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_ledger_view(item));
    }
    views
}

pub fn sum_silence_ledger_view(view: &SilenceLedgerView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_cost + (parsed_burst as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_ledger("code-343", "name-984");
        let view = to_silence_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
