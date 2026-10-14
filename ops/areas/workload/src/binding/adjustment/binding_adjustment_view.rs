use super::*;

use acme_kit::text;

/// How screens and exports show a binding adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAdjustmentView {
    pub code_text: String,
    pub name_text: String,
    pub fingerprint: String,
    pub discount: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_binding_adjustment_view(item: &BindingAdjustment) -> BindingAdjustmentView {
    BindingAdjustmentView {
        code_text: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 40),
        fingerprint: item.fingerprint.clone(),
        discount: format!("{:.2}", item.discount),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_adjustment_views(items: &[BindingAdjustment]) -> Vec<BindingAdjustmentView> {
    let mut views: Vec<BindingAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_adjustment_view(item));
    }
    views
}

pub fn sum_binding_adjustment_view(view: &BindingAdjustmentView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_discount + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_adjustment("code-466", "name-853");
        let view = to_binding_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
