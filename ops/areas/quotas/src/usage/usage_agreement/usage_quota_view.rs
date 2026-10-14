use super::*;

/// How screens and exports show a usage quota.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageQuotaView {
    pub code: String,
    pub title: String,
    pub batch_size_text: String,
    pub host: String,
    pub amount: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_usage_quota_view(item: &UsageQuota) -> UsageQuotaView {
    UsageQuotaView {
        code: item.code.clone(),
        title: item.title.clone(),
        batch_size_text: item.batch_size.to_string(),
        host: item.host.clone(),
        amount: format!("{:.2}", item.amount),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_quota_views(items: &[UsageQuota]) -> Vec<UsageQuotaView> {
    let mut views: Vec<UsageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_quota_view(item));
    }
    views
}

pub fn sum_usage_quota_view(view: &UsageQuotaView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_amount + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_quota("code-788", "title-690");
        let view = to_usage_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
