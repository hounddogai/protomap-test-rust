use super::*;

/// How screens and exports show a dns record mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingView {
    pub code: String,
    pub label: String,
    pub description_text: String,
    pub timezone: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_dns_record_mapping_view(item: &DnsRecordMapping) -> DnsRecordMappingView {
    DnsRecordMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        description_text: item.description.clone(),
        timezone: item.timezone.clone(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_mapping_views(items: &[DnsRecordMapping]) -> Vec<DnsRecordMappingView> {
    let mut views: Vec<DnsRecordMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_mapping_view(item));
    }
    views
}

pub fn sum_dns_record_mapping_view(view: &DnsRecordMappingView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_mapping("code-966", "label-568");
        let view = to_dns_record_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
