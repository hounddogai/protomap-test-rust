use super::*;

use acme_kit::text;

/// How screens and exports show a tenant notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantNoticeView {
    pub code: String,
    pub title: String,
    pub maximum: String,
    pub summary: String,
    pub segment: String,
    pub score_text: String,
    pub locale: String,
    pub created_at: String,
    pub owner_id: String,
    pub cost: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_tenant_notice_view(item: &TenantNotice) -> TenantNoticeView {
    TenantNoticeView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        maximum: format!("{:.2}", item.maximum),
        summary: item.summary.clone(),
        segment: item.segment.clone(),
        score_text: format!("{:.2}", item.score),
        locale: item.locale.clone(),
        created_at: item.created_at.to_string(),
        owner_id: item.owner_id.clone(),
        cost: format!("{:.2}", item.cost),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_notice_views(items: &[TenantNotice]) -> Vec<TenantNoticeView> {
    let mut views: Vec<TenantNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_notice_view(item));
    }
    views
}

pub fn sum_tenant_notice_view(view: &TenantNoticeView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_maximum + parsed_score + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_notice("code-499", "title-707");
        let view = to_tenant_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
