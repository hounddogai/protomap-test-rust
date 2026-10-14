use super::*;

/// How screens and exports show a dns record item.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordItemView {
    pub code: String,
    pub label: String,
    pub key_id_text: String,
    pub retries: String,
    pub rotation_days_text: String,
    pub grace_days: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_dns_record_item_view(item: &DnsRecordItem) -> DnsRecordItemView {
    DnsRecordItemView {
        code: item.code.clone(),
        label: item.label.clone(),
        key_id_text: item.key_id.clone(),
        retries: item.retries.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        grace_days: item.grace_days.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_item_views(items: &[DnsRecordItem]) -> Vec<DnsRecordItemView> {
    let mut views: Vec<DnsRecordItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_item_view(item));
    }
    views
}

pub fn sum_dns_record_item_view(view: &DnsRecordItemView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_rotation_days as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_item("code-974", "label-391");
        let view = to_dns_record_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
