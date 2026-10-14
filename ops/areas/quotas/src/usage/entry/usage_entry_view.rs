use super::*;

use acme_kit::text;

/// How screens and exports show a usage entry.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntryView {
    pub code: String,
    pub name_text: String,
    pub duration_minutes: String,
    pub external_ref_text: String,
    pub country: String,
    pub tenant_id_text: String,
    pub fingerprint_text: String,
    pub principal: String,
    pub visible: String,
    pub channel: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_usage_entry_view(item: &UsageEntry) -> UsageEntryView {
    UsageEntryView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        duration_minutes: item.duration_minutes.to_string(),
        external_ref_text: item.external_ref.clone(),
        country: item.country.clone(),
        tenant_id_text: item.tenant_id.clone(),
        fingerprint_text: item.fingerprint.clone(),
        principal: text::truncate(&item.principal, 40),
        visible: format!("{}", item.visible),
        channel: item.channel.clone(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_entry_views(items: &[UsageEntry]) -> Vec<UsageEntryView> {
    let mut views: Vec<UsageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_entry_view(item));
    }
    views
}

pub fn sum_usage_entry_view(view: &UsageEntryView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_duration_minutes as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_entry("code-871", "name-770");
        let view = to_usage_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
