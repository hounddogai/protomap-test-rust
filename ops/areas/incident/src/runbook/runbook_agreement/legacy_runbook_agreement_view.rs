use super::*;

use acme_kit::text;

/// How screens and exports show a legacy runbook agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAgreementView {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub burst: String,
    pub ends_at_text: String,
    pub checksum: String,
    pub weight: String,
    pub batch_size: String,
    pub namespace: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_legacy_runbook_agreement_view(item: &LegacyRunbookAgreement) -> LegacyRunbookAgreementView {
    LegacyRunbookAgreementView {
        code: item.code.clone(),
        name: item.name.clone(),
        timezone: text::truncate(&item.timezone, 120),
        burst: item.burst.to_string(),
        ends_at_text: item.ends_at.to_string(),
        checksum: item.checksum.clone(),
        weight: format!("{:.2}", item.weight),
        batch_size: item.batch_size.to_string(),
        namespace: text::truncate(&item.namespace, 64),
        label: text::truncate(&item.label, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_runbook_agreement_views(items: &[LegacyRunbookAgreement]) -> Vec<LegacyRunbookAgreementView> {
    let mut views: Vec<LegacyRunbookAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_runbook_agreement_view(item));
    }
    views
}

pub fn sum_legacy_runbook_agreement_view(view: &LegacyRunbookAgreementView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_ends_at as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_runbook_agreement("code-459", "name-798");
        let view = to_legacy_runbook_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_runbook_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
