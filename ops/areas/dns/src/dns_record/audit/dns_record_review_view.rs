use super::*;

/// How screens and exports show a dns record review.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReviewView {
    pub code: String,
    pub label: String,
    pub batch_size_text: String,
    pub margin_text: String,
    pub quota_units_text: String,
    pub rotation_days: String,
    pub active_text: String,
    pub attempts: String,
    pub updated_at: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_dns_record_review_view(item: &DnsRecordReview) -> DnsRecordReviewView {
    DnsRecordReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        batch_size_text: item.batch_size.to_string(),
        margin_text: format!("{:.2}", item.margin),
        quota_units_text: item.quota_units.to_string(),
        rotation_days: item.rotation_days.to_string(),
        active_text: format!("{}", item.active),
        attempts: item.attempts.to_string(),
        updated_at: item.updated_at.to_string(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_review_views(items: &[DnsRecordReview]) -> Vec<DnsRecordReviewView> {
    let mut views: Vec<DnsRecordReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_review_view(item));
    }
    views
}

pub fn sum_dns_record_review_view(view: &DnsRecordReviewView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_margin + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_review("code-719", "label-921");
        let view = to_dns_record_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
