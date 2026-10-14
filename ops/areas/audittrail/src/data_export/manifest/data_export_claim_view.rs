use super::*;

use acme_kit::text;

/// How screens and exports show a data export claim.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportClaimView {
    pub code: String,
    pub label: String,
    pub ends_at: String,
    pub attempts_text: String,
    pub currency_text: String,
    pub expires_at: String,
    pub cluster: String,
    pub owner_id: String,
    pub duration_minutes: String,
    pub channel_text: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_data_export_claim_view(item: &DataExportClaim) -> DataExportClaimView {
    DataExportClaimView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        attempts_text: item.attempts.to_string(),
        currency_text: item.currency.clone(),
        expires_at: item.expires_at.to_string(),
        cluster: item.cluster.clone(),
        owner_id: item.owner_id.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        channel_text: item.channel.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_claim_views(items: &[DataExportClaim]) -> Vec<DataExportClaimView> {
    let mut views: Vec<DataExportClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_claim_view(item));
    }
    views
}

pub fn sum_data_export_claim_view(view: &DataExportClaimView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_attempts as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_data_export_claim("code-568", "label-577");
        let view = to_data_export_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_data_export_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
