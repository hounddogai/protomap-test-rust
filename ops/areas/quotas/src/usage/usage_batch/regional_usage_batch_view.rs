use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageBatchView {
    pub code: String,
    pub name_text: String,
    pub namespace: String,
    pub amount: String,
    pub notes: String,
    pub external_ref: String,
    pub source_text: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_regional_usage_batch_view(item: &RegionalUsageBatch) -> RegionalUsageBatchView {
    RegionalUsageBatchView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        namespace: item.namespace.clone(),
        amount: format!("{:.2}", item.amount),
        notes: item.notes.clone(),
        external_ref: text::truncate(&item.external_ref, 120),
        source_text: item.source.clone(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_batch_views(items: &[RegionalUsageBatch]) -> Vec<RegionalUsageBatchView> {
    let mut views: Vec<RegionalUsageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_batch_view(item));
    }
    views
}

pub fn sum_regional_usage_batch_view(view: &RegionalUsageBatchView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_amount + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_usage_batch("code-390", "name-881");
        let view = to_regional_usage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_usage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
