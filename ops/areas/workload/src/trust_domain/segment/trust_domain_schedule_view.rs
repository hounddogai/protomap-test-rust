use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScheduleView {
    pub code: String,
    pub label: String,
    pub margin: String,
    pub region: String,
    pub archived: String,
    pub title: String,
    pub sequence: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_trust_domain_schedule_view(item: &TrustDomainSchedule) -> TrustDomainScheduleView {
    TrustDomainScheduleView {
        code: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 120),
        margin: format!("{:.2}", item.margin),
        region: item.region.clone(),
        archived: format!("{}", item.archived),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        phone: text::truncate(&item.phone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_schedule_views(items: &[TrustDomainSchedule]) -> Vec<TrustDomainScheduleView> {
    let mut views: Vec<TrustDomainScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_schedule_view(item));
    }
    views
}

pub fn sum_trust_domain_schedule_view(view: &TrustDomainScheduleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_margin + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_schedule("code-956", "label-480");
        let view = to_trust_domain_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
