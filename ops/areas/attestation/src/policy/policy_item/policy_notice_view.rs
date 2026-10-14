use super::*;

use acme_kit::text;

/// How screens and exports show a policy notice.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeView {
    pub code: String,
    pub title: String,
    pub retries: String,
    pub limit: String,
    pub region: String,
    pub approved_text: String,
    pub budget: String,
    pub key_id: String,
    pub tenant_id_text: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_policy_notice_view(item: &PolicyNotice) -> PolicyNoticeView {
    PolicyNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        retries: item.retries.to_string(),
        limit: item.limit.to_string(),
        region: text::truncate(&item.region, 40),
        approved_text: format!("{}", item.approved),
        budget: format!("{:.2}", item.budget),
        key_id: item.key_id.clone(),
        tenant_id_text: text::truncate(&item.tenant_id, 64),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_notice_views(items: &[PolicyNotice]) -> Vec<PolicyNoticeView> {
    let mut views: Vec<PolicyNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_notice_view(item));
    }
    views
}

pub fn sum_policy_notice_view(view: &PolicyNoticeView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_limit as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_notice("code-670", "title-769");
        let view = to_policy_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
