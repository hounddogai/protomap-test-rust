use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive suspension quota.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionQuotaView {
    pub code: String,
    pub label: String,
    pub ratio: String,
    pub pinned: String,
    pub amount: String,
    pub scope: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_adaptive_suspension_quota_view(item: &AdaptiveSuspensionQuota) -> AdaptiveSuspensionQuotaView {
    AdaptiveSuspensionQuotaView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        ratio: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        amount: format!("{:.2}", item.amount),
        scope: item.scope.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_suspension_quota_views(items: &[AdaptiveSuspensionQuota]) -> Vec<AdaptiveSuspensionQuotaView> {
    let mut views: Vec<AdaptiveSuspensionQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_suspension_quota_view(item));
    }
    views
}

pub fn sum_adaptive_suspension_quota_view(view: &AdaptiveSuspensionQuotaView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
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
    Ok(parsed_ratio + parsed_amount + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_suspension_quota("code-638", "label-80");
        let view = to_adaptive_suspension_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_suspension_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
