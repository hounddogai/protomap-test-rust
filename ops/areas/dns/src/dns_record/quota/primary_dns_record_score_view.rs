use super::*;

use acme_kit::text;

/// How screens and exports show a primary dns record score.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordScoreView {
    pub code: String,
    pub title_text: String,
    pub maximum_text: String,
    pub port_text: String,
    pub visible: String,
    pub external_ref: String,
    pub rotation_days_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_primary_dns_record_score_view(item: &PrimaryDnsRecordScore) -> PrimaryDnsRecordScoreView {
    PrimaryDnsRecordScoreView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        port_text: item.port.to_string(),
        visible: format!("{}", item.visible),
        external_ref: item.external_ref.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        phone: text::truncate(&item.phone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_dns_record_score_views(items: &[PrimaryDnsRecordScore]) -> Vec<PrimaryDnsRecordScoreView> {
    let mut views: Vec<PrimaryDnsRecordScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_dns_record_score_view(item));
    }
    views
}

pub fn sum_primary_dns_record_score_view(view: &PrimaryDnsRecordScoreView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_maximum + (parsed_port as f64) + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_dns_record_score("code-479", "title-81");
        let view = to_primary_dns_record_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_dns_record_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
