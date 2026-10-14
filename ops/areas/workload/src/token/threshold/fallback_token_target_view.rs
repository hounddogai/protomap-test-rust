use super::*;

use acme_kit::text;

/// How screens and exports show a fallback token target.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTargetView {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub limit_text: String,
    pub quantity_text: String,
    pub burst_text: String,
    pub fingerprint: String,
    pub grace_days_text: String,
    pub ttl_seconds: String,
    pub phone: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_fallback_token_target_view(item: &FallbackTokenTarget) -> FallbackTokenTargetView {
    FallbackTokenTargetView {
        code: item.code.clone(),
        name: item.name.clone(),
        principal: item.principal.clone(),
        limit_text: item.limit.to_string(),
        quantity_text: item.quantity.to_string(),
        burst_text: item.burst.to_string(),
        fingerprint: item.fingerprint.clone(),
        grace_days_text: item.grace_days.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        phone: text::truncate(&item.phone, 120),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_token_target_views(items: &[FallbackTokenTarget]) -> Vec<FallbackTokenTargetView> {
    let mut views: Vec<FallbackTokenTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_token_target_view(item));
    }
    views
}

pub fn sum_fallback_token_target_view(view: &FallbackTokenTargetView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_quantity as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_token_target("code-653", "name-927");
        let view = to_fallback_token_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_token_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
