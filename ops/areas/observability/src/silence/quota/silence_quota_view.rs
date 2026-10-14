use super::*;

use acme_kit::text;

/// How screens and exports show a silence quota.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaView {
    pub code: String,
    pub label: String,
    pub attempts: String,
    pub created_at: String,
    pub unit: String,
    pub rate_text: String,
    pub floor_value: String,
    pub cipher_text: String,
    pub country_text: String,
    pub amount_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_silence_quota_view(item: &SilenceQuota) -> SilenceQuotaView {
    SilenceQuotaView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        attempts: item.attempts.to_string(),
        created_at: item.created_at.to_string(),
        unit: item.unit.clone(),
        rate_text: format!("{:.2}", item.rate),
        floor_value: format!("{:.2}", item.floor_value),
        cipher_text: text::truncate(&item.cipher, 40),
        country_text: item.country.clone(),
        amount_text: format!("{:.2}", item.amount),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_quota_views(items: &[SilenceQuota]) -> Vec<SilenceQuotaView> {
    let mut views: Vec<SilenceQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_quota_view(item));
    }
    views
}

pub fn sum_silence_quota_view(view: &SilenceQuotaView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_created_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_quota("code-255", "label-531");
        let view = to_silence_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
