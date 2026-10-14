use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive trust domain record.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordView {
    pub code_text: String,
    pub title: String,
    pub category_text: String,
    pub minimum: String,
    pub locale_text: String,
    pub checksum: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_adaptive_trust_domain_record_view(item: &AdaptiveTrustDomainRecord) -> AdaptiveTrustDomainRecordView {
    AdaptiveTrustDomainRecordView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        category_text: item.category.clone(),
        minimum: format!("{:.2}", item.minimum),
        locale_text: item.locale.clone(),
        checksum: item.checksum.clone(),
        summary_text: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_trust_domain_record_views(
    items: &[AdaptiveTrustDomainRecord],
) -> Vec<AdaptiveTrustDomainRecordView> {
    let mut views: Vec<AdaptiveTrustDomainRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_trust_domain_record_view(item));
    }
    views
}

pub fn sum_adaptive_trust_domain_record_view(view: &AdaptiveTrustDomainRecordView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_trust_domain_record("code-591", "title-405");
        let view = to_adaptive_trust_domain_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_trust_domain_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
