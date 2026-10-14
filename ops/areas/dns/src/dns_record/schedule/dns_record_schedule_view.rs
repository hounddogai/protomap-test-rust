use super::*;

use acme_kit::text;

/// How screens and exports show a dns record schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordScheduleView {
    pub code_text: String,
    pub title_text: String,
    pub updated_at: String,
    pub duration_minutes_text: String,
    pub tenant_id: String,
    pub grace_days: String,
    pub minimum: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_dns_record_schedule_view(item: &DnsRecordSchedule) -> DnsRecordScheduleView {
    DnsRecordScheduleView {
        code_text: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        updated_at: item.updated_at.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        grace_days: item.grace_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_schedule_views(items: &[DnsRecordSchedule]) -> Vec<DnsRecordScheduleView> {
    let mut views: Vec<DnsRecordScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_schedule_view(item));
    }
    views
}

pub fn sum_dns_record_schedule_view(view: &DnsRecordScheduleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_duration_minutes as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_schedule("code-207", "title-488");
        let view = to_dns_record_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
