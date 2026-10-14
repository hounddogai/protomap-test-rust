use super::*;

use acme_kit::text;

/// How screens and exports show a lenient dns record baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordBaselineView {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub capacity_text: String,
    pub summary: String,
    pub duration_minutes: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_lenient_dns_record_baseline_view(item: &LenientDnsRecordBaseline) -> LenientDnsRecordBaselineView {
    LenientDnsRecordBaselineView {
        code: item.code.clone(),
        label: item.label.clone(),
        owner_id: text::truncate(&item.owner_id, 80),
        capacity_text: item.capacity.to_string(),
        summary: item.summary.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_dns_record_baseline_views(items: &[LenientDnsRecordBaseline]) -> Vec<LenientDnsRecordBaselineView> {
    let mut views: Vec<LenientDnsRecordBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_dns_record_baseline_view(item));
    }
    views
}

pub fn sum_lenient_dns_record_baseline_view(view: &LenientDnsRecordBaselineView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_dns_record_baseline("code-589", "label-957");
        let view = to_lenient_dns_record_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_dns_record_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
