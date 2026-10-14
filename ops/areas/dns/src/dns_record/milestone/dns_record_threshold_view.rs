use super::*;

/// How screens and exports show a dns record threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdView {
    pub code: String,
    pub title: String,
    pub ceiling: String,
    pub margin_text: String,
    pub starts_at: String,
    pub quantity: String,
    pub grace_days: String,
    pub key_id: String,
    pub port_text: String,
    pub priority: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_dns_record_threshold_view(item: &DnsRecordThreshold) -> DnsRecordThresholdView {
    DnsRecordThresholdView {
        code: item.code.clone(),
        title: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        margin_text: format!("{:.2}", item.margin),
        starts_at: item.starts_at.to_string(),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        key_id: item.key_id.clone(),
        port_text: item.port.to_string(),
        priority: item.priority.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_threshold_views(items: &[DnsRecordThreshold]) -> Vec<DnsRecordThresholdView> {
    let mut views: Vec<DnsRecordThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_threshold_view(item));
    }
    views
}

pub fn sum_dns_record_threshold_view(view: &DnsRecordThresholdView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_ceiling + parsed_margin + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_threshold("code-747", "title-661");
        let view = to_dns_record_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
