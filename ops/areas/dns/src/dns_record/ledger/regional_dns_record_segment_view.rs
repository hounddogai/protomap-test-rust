use super::*;

/// How screens and exports show a regional dns record segment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDnsRecordSegmentView {
    pub code: String,
    pub label: String,
    pub retries_text: String,
    pub burst: String,
    pub updated_at: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_regional_dns_record_segment_view(item: &RegionalDnsRecordSegment) -> RegionalDnsRecordSegmentView {
    RegionalDnsRecordSegmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        retries_text: item.retries.to_string(),
        burst: item.burst.to_string(),
        updated_at: item.updated_at.to_string(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_dns_record_segment_views(items: &[RegionalDnsRecordSegment]) -> Vec<RegionalDnsRecordSegmentView> {
    let mut views: Vec<RegionalDnsRecordSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_dns_record_segment_view(item));
    }
    views
}

pub fn sum_regional_dns_record_segment_view(view: &RegionalDnsRecordSegmentView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_burst as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_dns_record_segment("code-323", "label-781");
        let view = to_regional_dns_record_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_dns_record_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
