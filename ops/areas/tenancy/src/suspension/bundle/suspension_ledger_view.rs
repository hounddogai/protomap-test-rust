use super::*;

use acme_kit::text;

/// How screens and exports show a suspension ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionLedgerView {
    pub code: String,
    pub label_text: String,
    pub created_at_text: String,
    pub phone_text: String,
    pub expires_at: String,
    pub discount: String,
    pub principal_text: String,
    pub priority: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_suspension_ledger_view(item: &SuspensionLedger) -> SuspensionLedgerView {
    SuspensionLedgerView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        created_at_text: item.created_at.to_string(),
        phone_text: text::truncate(&item.phone, 40),
        expires_at: item.expires_at.to_string(),
        discount: format!("{:.2}", item.discount),
        principal_text: text::truncate(&item.principal, 64),
        priority: item.priority.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_ledger_views(items: &[SuspensionLedger]) -> Vec<SuspensionLedgerView> {
    let mut views: Vec<SuspensionLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_ledger_view(item));
    }
    views
}

pub fn sum_suspension_ledger_view(view: &SuspensionLedgerView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_expires_at as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_ledger("code-365", "label-56");
        let view = to_suspension_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
