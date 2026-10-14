use super::*;

use acme_kit::text;

/// How screens and exports show a fallback approval entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntryView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub rate: String,
    pub principal_text: String,
    pub summary: String,
    pub tenant_id: String,
    pub locale: String,
    pub region: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_fallback_approval_entry_view(item: &FallbackApprovalEntry) -> FallbackApprovalEntryView {
    FallbackApprovalEntryView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        amount: format!("{:.2}", item.amount),
        rate: format!("{:.2}", item.rate),
        principal_text: text::truncate(&item.principal, 40),
        summary: item.summary.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        locale: item.locale.clone(),
        region: item.region.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_approval_entry_views(items: &[FallbackApprovalEntry]) -> Vec<FallbackApprovalEntryView> {
    let mut views: Vec<FallbackApprovalEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_approval_entry_view(item));
    }
    views
}

pub fn sum_fallback_approval_entry_view(view: &FallbackApprovalEntryView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_amount + parsed_rate + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_approval_entry("code-586", "label-852");
        let view = to_fallback_approval_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_approval_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
