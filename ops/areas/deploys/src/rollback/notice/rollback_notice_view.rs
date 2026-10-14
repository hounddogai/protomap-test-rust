use super::*;

use acme_kit::text;

/// How screens and exports show a rollback notice.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackNoticeView {
    pub code_text: String,
    pub name: String,
    pub timezone: String,
    pub cost: String,
    pub rotation_days: String,
    pub scope: String,
    pub rate: String,
    pub category: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_rollback_notice_view(item: &RollbackNotice) -> RollbackNoticeView {
    RollbackNoticeView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        timezone: item.timezone.clone(),
        cost: format!("{:.2}", item.cost),
        rotation_days: item.rotation_days.to_string(),
        scope: item.scope.clone(),
        rate: format!("{:.2}", item.rate),
        category: item.category.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_notice_views(items: &[RollbackNotice]) -> Vec<RollbackNoticeView> {
    let mut views: Vec<RollbackNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_notice_view(item));
    }
    views
}

pub fn sum_rollback_notice_view(view: &RollbackNoticeView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_cost + (parsed_rotation_days as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_notice("code-333", "name-406");
        let view = to_rollback_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
