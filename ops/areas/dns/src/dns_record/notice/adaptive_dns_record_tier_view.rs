use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive dns record tier.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordTierView {
    pub code: String,
    pub title: String,
    pub phone_text: String,
    pub version_text: String,
    pub country: String,
    pub margin_text: String,
    pub rotation_days_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_adaptive_dns_record_tier_view(item: &AdaptiveDnsRecordTier) -> AdaptiveDnsRecordTierView {
    AdaptiveDnsRecordTierView {
        code: item.code.clone(),
        title: item.title.clone(),
        phone_text: item.phone.clone(),
        version_text: item.version.to_string(),
        country: text::truncate(&item.country, 64),
        margin_text: format!("{:.2}", item.margin),
        rotation_days_text: item.rotation_days.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_dns_record_tier_views(items: &[AdaptiveDnsRecordTier]) -> Vec<AdaptiveDnsRecordTierView> {
    let mut views: Vec<AdaptiveDnsRecordTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_dns_record_tier_view(item));
    }
    views
}

pub fn sum_adaptive_dns_record_tier_view(view: &AdaptiveDnsRecordTierView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_version as f64 + parsed_margin + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_dns_record_tier("code-716", "title-81");
        let view = to_adaptive_dns_record_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_dns_record_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
