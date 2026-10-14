use super::*;

/// How screens and exports show a rollback adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentView {
    pub code_text: String,
    pub label_text: String,
    pub expires_at: String,
    pub floor_value_text: String,
    pub capacity: String,
    pub quantity: String,
    pub score_text: String,
    pub principal: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_rollback_adjustment_view(item: &RollbackAdjustment) -> RollbackAdjustmentView {
    RollbackAdjustmentView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        capacity: item.capacity.to_string(),
        quantity: item.quantity.to_string(),
        score_text: format!("{:.2}", item.score),
        principal: item.principal.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_adjustment_views(items: &[RollbackAdjustment]) -> Vec<RollbackAdjustmentView> {
    let mut views: Vec<RollbackAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_adjustment_view(item));
    }
    views
}

pub fn sum_rollback_adjustment_view(view: &RollbackAdjustmentView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_floor_value + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_adjustment("code-21", "label-948");
        let view = to_rollback_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
