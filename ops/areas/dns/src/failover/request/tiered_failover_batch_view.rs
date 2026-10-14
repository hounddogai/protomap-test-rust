use super::*;

use acme_kit::text;

/// How screens and exports show a tiered failover batch.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverBatchView {
    pub code_text: String,
    pub label: String,
    pub external_ref: String,
    pub rate: String,
    pub owner_id: String,
    pub currency: String,
    pub duration_minutes_text: String,
    pub key_id: String,
    pub burst: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_tiered_failover_batch_view(item: &TieredFailoverBatch) -> TieredFailoverBatchView {
    TieredFailoverBatchView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        external_ref: item.external_ref.clone(),
        rate: format!("{:.2}", item.rate),
        owner_id: item.owner_id.clone(),
        currency: item.currency.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        key_id: item.key_id.clone(),
        burst: item.burst.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_failover_batch_views(items: &[TieredFailoverBatch]) -> Vec<TieredFailoverBatchView> {
    let mut views: Vec<TieredFailoverBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_failover_batch_view(item));
    }
    views
}

pub fn sum_tiered_failover_batch_view(view: &TieredFailoverBatchView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_rate + (parsed_duration_minutes as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_failover_batch("code-169", "label-505");
        let view = to_tiered_failover_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_failover_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
