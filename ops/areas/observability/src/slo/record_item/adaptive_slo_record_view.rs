use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive slo record.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloRecordView {
    pub code_text: String,
    pub name: String,
    pub channel_text: String,
    pub grace_days: String,
    pub timezone: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_adaptive_slo_record_view(item: &AdaptiveSloRecord) -> AdaptiveSloRecordView {
    AdaptiveSloRecordView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        channel_text: item.channel.clone(),
        grace_days: item.grace_days.to_string(),
        timezone: item.timezone.clone(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_slo_record_views(items: &[AdaptiveSloRecord]) -> Vec<AdaptiveSloRecordView> {
    let mut views: Vec<AdaptiveSloRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_slo_record_view(item));
    }
    views
}

pub fn sum_adaptive_slo_record_view(view: &AdaptiveSloRecordView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_slo_record("code-954", "name-229");
        let view = to_adaptive_slo_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_slo_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
