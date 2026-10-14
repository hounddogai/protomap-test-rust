use super::*;

use acme_kit::text;

/// How screens and exports show a batched suspension tier.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionTierView {
    pub code: String,
    pub name: String,
    pub port_text: String,
    pub ends_at: String,
    pub approved_text: String,
    pub description_text: String,
    pub batch_size_text: String,
    pub notes: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_batched_suspension_tier_view(item: &BatchedSuspensionTier) -> BatchedSuspensionTierView {
    BatchedSuspensionTierView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        port_text: item.port.to_string(),
        ends_at: item.ends_at.to_string(),
        approved_text: format!("{}", item.approved),
        description_text: item.description.clone(),
        batch_size_text: item.batch_size.to_string(),
        notes: item.notes.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_suspension_tier_views(items: &[BatchedSuspensionTier]) -> Vec<BatchedSuspensionTierView> {
    let mut views: Vec<BatchedSuspensionTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_suspension_tier_view(item));
    }
    views
}

pub fn sum_batched_suspension_tier_view(view: &BatchedSuspensionTierView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_port as f64 + (parsed_ends_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_suspension_tier("code-436", "name-7");
        let view = to_batched_suspension_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_suspension_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
