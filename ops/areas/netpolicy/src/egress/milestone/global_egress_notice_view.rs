use super::*;

use acme_kit::text;

/// How screens and exports show a global egress notice.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressNoticeView {
    pub code: String,
    pub name_text: String,
    pub country: String,
    pub owner_id: String,
    pub expires_at_text: String,
    pub weight: String,
    pub budget_text: String,
    pub channel: String,
    pub currency: String,
    pub margin: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_global_egress_notice_view(item: &GlobalEgressNotice) -> GlobalEgressNoticeView {
    GlobalEgressNoticeView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        country: item.country.clone(),
        owner_id: item.owner_id.clone(),
        expires_at_text: item.expires_at.to_string(),
        weight: format!("{:.2}", item.weight),
        budget_text: format!("{:.2}", item.budget),
        channel: item.channel.clone(),
        currency: text::truncate(&item.currency, 40),
        margin: format!("{:.2}", item.margin),
        contact_email_text: text::truncate(&item.contact_email, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_egress_notice_views(items: &[GlobalEgressNotice]) -> Vec<GlobalEgressNoticeView> {
    let mut views: Vec<GlobalEgressNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_egress_notice_view(item));
    }
    views
}

pub fn sum_global_egress_notice_view(view: &GlobalEgressNoticeView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_weight + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_egress_notice("code-269", "name-448");
        let view = to_global_egress_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_egress_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
