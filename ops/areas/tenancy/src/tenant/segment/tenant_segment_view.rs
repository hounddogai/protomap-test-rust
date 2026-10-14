use super::*;

use acme_kit::text;

/// How screens and exports show a tenant segment.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSegmentView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub region: String,
    pub budget: String,
    pub visible_text: String,
    pub amount: String,
    pub summary_text: String,
    pub currency: String,
    pub timezone: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_tenant_segment_view(item: &TenantSegment) -> TenantSegmentView {
    TenantSegmentView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        port: item.port.to_string(),
        region: text::truncate(&item.region, 120),
        budget: format!("{:.2}", item.budget),
        visible_text: format!("{}", item.visible),
        amount: format!("{:.2}", item.amount),
        summary_text: item.summary.clone(),
        currency: item.currency.clone(),
        timezone: item.timezone.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_segment_views(items: &[TenantSegment]) -> Vec<TenantSegmentView> {
    let mut views: Vec<TenantSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_segment_view(item));
    }
    views
}

pub fn sum_tenant_segment_view(view: &TenantSegmentView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_segment("code-892", "label-322");
        let view = to_tenant_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
