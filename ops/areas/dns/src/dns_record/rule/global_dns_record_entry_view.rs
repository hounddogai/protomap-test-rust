use super::*;

use acme_kit::text;

/// How screens and exports show a global dns record entry.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordEntryView {
    pub code: String,
    pub title: String,
    pub created_at: String,
    pub rotation_days: String,
    pub port: String,
    pub budget: String,
    pub country: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_global_dns_record_entry_view(item: &GlobalDnsRecordEntry) -> GlobalDnsRecordEntryView {
    GlobalDnsRecordEntryView {
        code: item.code.clone(),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        port: item.port.to_string(),
        budget: format!("{:.2}", item.budget),
        country: text::truncate(&item.country, 120),
        contact_email: text::truncate(&item.contact_email, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_dns_record_entry_views(items: &[GlobalDnsRecordEntry]) -> Vec<GlobalDnsRecordEntryView> {
    let mut views: Vec<GlobalDnsRecordEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_dns_record_entry_view(item));
    }
    views
}

pub fn sum_global_dns_record_entry_view(view: &GlobalDnsRecordEntryView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_rotation_days as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_dns_record_entry("code-971", "title-17");
        let view = to_global_dns_record_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_dns_record_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
