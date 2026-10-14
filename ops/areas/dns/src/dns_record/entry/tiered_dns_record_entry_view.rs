use super::*;

/// How screens and exports show a tiered dns record entry.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordEntryView {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub window_days: String,
    pub phone: String,
    pub batch_size: String,
    pub retries: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_tiered_dns_record_entry_view(item: &TieredDnsRecordEntry) -> TieredDnsRecordEntryView {
    TieredDnsRecordEntryView {
        code: item.code.clone(),
        name: item.name.clone(),
        timezone: item.timezone.clone(),
        window_days: item.window_days.to_string(),
        phone: item.phone.clone(),
        batch_size: item.batch_size.to_string(),
        retries: item.retries.to_string(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_dns_record_entry_views(items: &[TieredDnsRecordEntry]) -> Vec<TieredDnsRecordEntryView> {
    let mut views: Vec<TieredDnsRecordEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_dns_record_entry_view(item));
    }
    views
}

pub fn sum_tiered_dns_record_entry_view(view: &TieredDnsRecordEntryView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_batch_size as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_dns_record_entry("code-207", "name-532");
        let view = to_tiered_dns_record_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_dns_record_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
