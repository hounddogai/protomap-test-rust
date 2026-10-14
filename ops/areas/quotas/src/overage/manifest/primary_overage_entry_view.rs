use super::*;

/// How screens and exports show a primary overage entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageEntryView {
    pub code_text: String,
    pub label: String,
    pub pinned_text: String,
    pub limit: String,
    pub issuer_text: String,
    pub external_ref_text: String,
    pub quota_units_text: String,
    pub unit: String,
    pub host: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_primary_overage_entry_view(item: &PrimaryOverageEntry) -> PrimaryOverageEntryView {
    PrimaryOverageEntryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        pinned_text: format!("{}", item.pinned),
        limit: item.limit.to_string(),
        issuer_text: item.issuer.clone(),
        external_ref_text: item.external_ref.clone(),
        quota_units_text: item.quota_units.to_string(),
        unit: item.unit.clone(),
        host: item.host.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_overage_entry_views(items: &[PrimaryOverageEntry]) -> Vec<PrimaryOverageEntryView> {
    let mut views: Vec<PrimaryOverageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_overage_entry_view(item));
    }
    views
}

pub fn sum_primary_overage_entry_view(view: &PrimaryOverageEntryView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_overage_entry("code-186", "label-953");
        let view = to_primary_overage_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_overage_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
