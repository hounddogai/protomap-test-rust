use super::*;

use acme_kit::text;

/// How screens and exports show a rolling dns record item.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItemView {
    pub code: String,
    pub label: String,
    pub archived_text: String,
    pub burst: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_rolling_dns_record_item_view(item: &RollingDnsRecordItem) -> RollingDnsRecordItemView {
    RollingDnsRecordItemView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        archived_text: format!("{}", item.archived),
        burst: item.burst.to_string(),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_dns_record_item_views(items: &[RollingDnsRecordItem]) -> Vec<RollingDnsRecordItemView> {
    let mut views: Vec<RollingDnsRecordItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_dns_record_item_view(item));
    }
    views
}

pub fn sum_rolling_dns_record_item_view(view: &RollingDnsRecordItemView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_dns_record_item("code-27", "label-353");
        let view = to_rolling_dns_record_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_dns_record_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
