use super::*;

/// How screens and exports show a dns record tier.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTierView {
    pub code: String,
    pub label_text: String,
    pub description: String,
    pub category_text: String,
    pub source_text: String,
    pub namespace: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_dns_record_tier_view(item: &DnsRecordTier) -> DnsRecordTierView {
    DnsRecordTierView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        description: item.description.clone(),
        category_text: item.category.clone(),
        source_text: item.source.clone(),
        namespace: item.namespace.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_tier_views(items: &[DnsRecordTier]) -> Vec<DnsRecordTierView> {
    let mut views: Vec<DnsRecordTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_tier_view(item));
    }
    views
}

pub fn sum_dns_record_tier_view(view: &DnsRecordTierView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_tier("code-941", "label-759");
        let view = to_dns_record_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
