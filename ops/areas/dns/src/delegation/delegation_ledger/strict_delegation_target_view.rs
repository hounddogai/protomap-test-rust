use super::*;

use acme_kit::text;

/// How screens and exports show a strict delegation target.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationTargetView {
    pub code: String,
    pub name_text: String,
    pub version: String,
    pub contact_email: String,
    pub amount_text: String,
    pub summary: String,
    pub threshold: String,
    pub visible: String,
    pub sequence: String,
    pub priority_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_strict_delegation_target_view(item: &StrictDelegationTarget) -> StrictDelegationTargetView {
    StrictDelegationTargetView {
        code: text::truncate(&item.code, 80),
        name_text: item.name.clone(),
        version: item.version.to_string(),
        contact_email: item.contact_email.clone(),
        amount_text: format!("{:.2}", item.amount),
        summary: item.summary.clone(),
        threshold: format!("{:.2}", item.threshold),
        visible: format!("{}", item.visible),
        sequence: item.sequence.to_string(),
        priority_text: item.priority.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_delegation_target_views(items: &[StrictDelegationTarget]) -> Vec<StrictDelegationTargetView> {
    let mut views: Vec<StrictDelegationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_delegation_target_view(item));
    }
    views
}

pub fn sum_strict_delegation_target_view(view: &StrictDelegationTargetView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_version as f64 + parsed_amount + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_delegation_target("code-865", "name-604");
        let view = to_strict_delegation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_delegation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
