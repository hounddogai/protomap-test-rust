use super::*;

/// How screens and exports show a pending action item target.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub burst: String,
    pub updated_at_text: String,
    pub cipher: String,
    pub ttl_seconds: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_pending_action_item_target_view(item: &PendingActionItemTarget) -> PendingActionItemTargetView {
    PendingActionItemTargetView {
        code: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        burst: item.burst.to_string(),
        updated_at_text: item.updated_at.to_string(),
        cipher: item.cipher.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_action_item_target_views(items: &[PendingActionItemTarget]) -> Vec<PendingActionItemTargetView> {
    let mut views: Vec<PendingActionItemTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_action_item_target_view(item));
    }
    views
}

pub fn sum_pending_action_item_target_view(view: &PendingActionItemTargetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_updated_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_action_item_target("code-22", "title-999");
        let view = to_pending_action_item_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_action_item_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
