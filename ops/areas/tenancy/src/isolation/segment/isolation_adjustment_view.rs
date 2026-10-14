use super::*;

use acme_kit::text;

/// How screens and exports show a isolation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationAdjustmentView {
    pub code: String,
    pub title_text: String,
    pub audience: String,
    pub external_ref: String,
    pub priority: String,
    pub approved: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_isolation_adjustment_view(item: &IsolationAdjustment) -> IsolationAdjustmentView {
    IsolationAdjustmentView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 64),
        audience: item.audience.clone(),
        external_ref: item.external_ref.clone(),
        priority: item.priority.to_string(),
        approved: format!("{}", item.approved),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_adjustment_views(items: &[IsolationAdjustment]) -> Vec<IsolationAdjustmentView> {
    let mut views: Vec<IsolationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_adjustment_view(item));
    }
    views
}

pub fn sum_isolation_adjustment_view(view: &IsolationAdjustmentView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_adjustment("code-210", "title-754");
        let view = to_isolation_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
