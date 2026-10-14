use super::*;

use acme_kit::text;

/// How screens and exports show a tiered overage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatchView {
    pub code_text: String,
    pub name_text: String,
    pub currency: String,
    pub margin: String,
    pub scope: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_tiered_overage_batch_view(item: &TieredOverageBatch) -> TieredOverageBatchView {
    TieredOverageBatchView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        currency: text::truncate(&item.currency, 64),
        margin: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_overage_batch_views(items: &[TieredOverageBatch]) -> Vec<TieredOverageBatchView> {
    let mut views: Vec<TieredOverageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_overage_batch_view(item));
    }
    views
}

pub fn sum_tiered_overage_batch_view(view: &TieredOverageBatchView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_overage_batch("code-482", "name-306");
        let view = to_tiered_overage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_overage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
