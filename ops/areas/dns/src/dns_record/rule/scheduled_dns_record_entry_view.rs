use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled dns record entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordEntryView {
    pub code_text: String,
    pub name_text: String,
    pub grace_days: String,
    pub margin: String,
    pub amount: String,
    pub rate_text: String,
    pub burst: String,
    pub created_at: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_scheduled_dns_record_entry_view(item: &ScheduledDnsRecordEntry) -> ScheduledDnsRecordEntryView {
    ScheduledDnsRecordEntryView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        grace_days: item.grace_days.to_string(),
        margin: format!("{:.2}", item.margin),
        amount: format!("{:.2}", item.amount),
        rate_text: format!("{:.2}", item.rate),
        burst: item.burst.to_string(),
        created_at: item.created_at.to_string(),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_dns_record_entry_views(items: &[ScheduledDnsRecordEntry]) -> Vec<ScheduledDnsRecordEntryView> {
    let mut views: Vec<ScheduledDnsRecordEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_dns_record_entry_view(item));
    }
    views
}

pub fn sum_scheduled_dns_record_entry_view(view: &ScheduledDnsRecordEntryView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_margin + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_dns_record_entry("code-270", "name-131");
        let view = to_scheduled_dns_record_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_dns_record_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
