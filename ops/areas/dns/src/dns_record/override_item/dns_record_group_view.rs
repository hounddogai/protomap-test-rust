use super::*;

/// How screens and exports show a dns record group.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordGroupView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub weight: String,
    pub starts_at_text: String,
    pub port: String,
    pub pinned: String,
    pub discount_text: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_dns_record_group_view(item: &DnsRecordGroup) -> DnsRecordGroupView {
    DnsRecordGroupView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        weight: format!("{:.2}", item.weight),
        starts_at_text: item.starts_at.to_string(),
        port: item.port.to_string(),
        pinned: format!("{}", item.pinned),
        discount_text: format!("{:.2}", item.discount),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_group_views(items: &[DnsRecordGroup]) -> Vec<DnsRecordGroupView> {
    let mut views: Vec<DnsRecordGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_group_view(item));
    }
    views
}

pub fn sum_dns_record_group_view(view: &DnsRecordGroupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_weight + (parsed_starts_at as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_group("code-643", "title-964");
        let view = to_dns_record_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
