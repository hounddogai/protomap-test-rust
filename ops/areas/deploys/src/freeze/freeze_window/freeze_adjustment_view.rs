use super::*;

use acme_kit::text;

/// How screens and exports show a freeze adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAdjustmentView {
    pub code: String,
    pub title_text: String,
    pub quantity: String,
    pub port: String,
    pub principal: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_freeze_adjustment_view(item: &FreezeAdjustment) -> FreezeAdjustmentView {
    FreezeAdjustmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quantity: item.quantity.to_string(),
        port: item.port.to_string(),
        principal: text::truncate(&item.principal, 120),
        name: text::truncate(&item.name, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_adjustment_views(items: &[FreezeAdjustment]) -> Vec<FreezeAdjustmentView> {
    let mut views: Vec<FreezeAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_adjustment_view(item));
    }
    views
}

pub fn sum_freeze_adjustment_view(view: &FreezeAdjustmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_adjustment("code-612", "title-71");
        let view = to_freeze_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
