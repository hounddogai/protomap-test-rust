use super::*;

use acme_kit::text;

/// How screens and exports show a fallback dns record allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDnsRecordAllocationView {
    pub code: String,
    pub name_text: String,
    pub version: String,
    pub minimum_text: String,
    pub margin: String,
    pub weight: String,
    pub maximum_text: String,
    pub ceiling: String,
    pub approved: String,
    pub updated_at_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_fallback_dns_record_allocation_view(item: &FallbackDnsRecordAllocation) -> FallbackDnsRecordAllocationView {
    FallbackDnsRecordAllocationView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        version: item.version.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        margin: format!("{:.2}", item.margin),
        weight: format!("{:.2}", item.weight),
        maximum_text: format!("{:.2}", item.maximum),
        ceiling: format!("{:.2}", item.ceiling),
        approved: format!("{}", item.approved),
        updated_at_text: item.updated_at.to_string(),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_dns_record_allocation_views(
    items: &[FallbackDnsRecordAllocation],
) -> Vec<FallbackDnsRecordAllocationView> {
    let mut views: Vec<FallbackDnsRecordAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_dns_record_allocation_view(item));
    }
    views
}

pub fn sum_fallback_dns_record_allocation_view(view: &FallbackDnsRecordAllocationView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_version as f64 + parsed_minimum + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_dns_record_allocation("code-350", "name-784");
        let view = to_fallback_dns_record_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_dns_record_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
