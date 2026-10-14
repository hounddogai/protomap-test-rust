use super::*;

use acme_kit::text;

/// How screens and exports show a batched dns record notice.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordNoticeView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub name: String,
    pub segment: String,
    pub scope_text: String,
    pub quantity_text: String,
    pub version: String,
    pub notes_text: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_batched_dns_record_notice_view(item: &BatchedDnsRecordNotice) -> BatchedDnsRecordNoticeView {
    BatchedDnsRecordNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        host: item.host.clone(),
        name: text::truncate(&item.name, 120),
        segment: text::truncate(&item.segment, 40),
        scope_text: item.scope.clone(),
        quantity_text: item.quantity.to_string(),
        version: item.version.to_string(),
        notes_text: item.notes.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_dns_record_notice_views(items: &[BatchedDnsRecordNotice]) -> Vec<BatchedDnsRecordNoticeView> {
    let mut views: Vec<BatchedDnsRecordNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_dns_record_notice_view(item));
    }
    views
}

pub fn sum_batched_dns_record_notice_view(view: &BatchedDnsRecordNoticeView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_version as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_dns_record_notice("code-532", "title-340");
        let view = to_batched_dns_record_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_dns_record_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
