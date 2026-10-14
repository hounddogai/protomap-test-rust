use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyNoticeView {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub key_id: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_ttl_policy_notice_view(item: &TtlPolicyNotice) -> TtlPolicyNoticeView {
    TtlPolicyNoticeView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        segment: item.segment.clone(),
        key_id: text::truncate(&item.key_id, 120),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_notice_views(items: &[TtlPolicyNotice]) -> Vec<TtlPolicyNoticeView> {
    let mut views: Vec<TtlPolicyNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_notice_view(item));
    }
    views
}

pub fn sum_ttl_policy_notice_view(view: &TtlPolicyNoticeView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_notice("code-417", "name-344");
        let view = to_ttl_policy_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
