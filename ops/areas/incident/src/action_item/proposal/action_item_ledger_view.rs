use super::*;

/// How screens and exports show a action item ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemLedgerView {
    pub code: String,
    pub title_text: String,
    pub principal: String,
    pub source: String,
    pub issuer: String,
    pub quota_units_text: String,
    pub country_text: String,
    pub notes_text: String,
    pub port_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_action_item_ledger_view(item: &ActionItemLedger) -> ActionItemLedgerView {
    ActionItemLedgerView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        principal: item.principal.clone(),
        source: item.source.clone(),
        issuer: item.issuer.clone(),
        quota_units_text: item.quota_units.to_string(),
        country_text: item.country.clone(),
        notes_text: item.notes.clone(),
        port_text: item.port.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_ledger_views(items: &[ActionItemLedger]) -> Vec<ActionItemLedgerView> {
    let mut views: Vec<ActionItemLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_ledger_view(item));
    }
    views
}

pub fn sum_action_item_ledger_view(view: &ActionItemLedgerView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_port as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_ledger("code-233", "title-677");
        let view = to_action_item_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
