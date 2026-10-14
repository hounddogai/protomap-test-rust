use super::*;

use acme_kit::text;

/// How screens and exports show a rolling binding adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingAdjustmentView {
    pub code: String,
    pub title_text: String,
    pub phone: String,
    pub locale: String,
    pub retries: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_rolling_binding_adjustment_view(item: &RollingBindingAdjustment) -> RollingBindingAdjustmentView {
    RollingBindingAdjustmentView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 64),
        phone: text::truncate(&item.phone, 80),
        locale: item.locale.clone(),
        retries: item.retries.to_string(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_binding_adjustment_views(items: &[RollingBindingAdjustment]) -> Vec<RollingBindingAdjustmentView> {
    let mut views: Vec<RollingBindingAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_binding_adjustment_view(item));
    }
    views
}

pub fn sum_rolling_binding_adjustment_view(view: &RollingBindingAdjustmentView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_binding_adjustment("code-621", "title-777");
        let view = to_rolling_binding_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_binding_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
