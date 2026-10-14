use super::*;

/// How screens and exports show a legacy binding batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingBatchView {
    pub code_text: String,
    pub name_text: String,
    pub cipher: String,
    pub quantity_text: String,
    pub window_days_text: String,
    pub capacity: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_legacy_binding_batch_view(item: &LegacyBindingBatch) -> LegacyBindingBatchView {
    LegacyBindingBatchView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        cipher: item.cipher.clone(),
        quantity_text: item.quantity.to_string(),
        window_days_text: item.window_days.to_string(),
        capacity: item.capacity.to_string(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_binding_batch_views(items: &[LegacyBindingBatch]) -> Vec<LegacyBindingBatchView> {
    let mut views: Vec<LegacyBindingBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_binding_batch_view(item));
    }
    views
}

pub fn sum_legacy_binding_batch_view(view: &LegacyBindingBatchView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_window_days as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_binding_batch("code-708", "name-331");
        let view = to_legacy_binding_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_binding_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
