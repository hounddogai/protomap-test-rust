use super::*;

use acme_kit::text;

/// How screens and exports show a global suspension batch.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatchView {
    pub code: String,
    pub label: String,
    pub external_ref_text: String,
    pub weight: String,
    pub active: String,
    pub retries: String,
    pub maximum: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_global_suspension_batch_view(item: &GlobalSuspensionBatch) -> GlobalSuspensionBatchView {
    GlobalSuspensionBatchView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 120),
        external_ref_text: text::truncate(&item.external_ref, 120),
        weight: format!("{:.2}", item.weight),
        active: format!("{}", item.active),
        retries: item.retries.to_string(),
        maximum: format!("{:.2}", item.maximum),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_suspension_batch_views(items: &[GlobalSuspensionBatch]) -> Vec<GlobalSuspensionBatchView> {
    let mut views: Vec<GlobalSuspensionBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_suspension_batch_view(item));
    }
    views
}

pub fn sum_global_suspension_batch_view(view: &GlobalSuspensionBatchView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_weight + (parsed_retries as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_suspension_batch("code-633", "label-28");
        let view = to_global_suspension_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_suspension_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
