use super::*;

/// How screens and exports show a usage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchView {
    pub code_text: String,
    pub name: String,
    pub visible_text: String,
    pub timezone: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_usage_batch_view(item: &UsageBatch) -> UsageBatchView {
    UsageBatchView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        visible_text: format!("{}", item.visible),
        timezone: item.timezone.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_batch_views(items: &[UsageBatch]) -> Vec<UsageBatchView> {
    let mut views: Vec<UsageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_batch_view(item));
    }
    views
}

pub fn sum_usage_batch_view(view: &UsageBatchView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_batch("code-657", "name-820");
        let view = to_usage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
