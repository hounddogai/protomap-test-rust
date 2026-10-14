use super::*;

/// How screens and exports show a rollout digest.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigestView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub cipher: String,
    pub starts_at: String,
    pub label: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_rollout_digest_view(item: &RolloutDigest) -> RolloutDigestView {
    RolloutDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        cipher: item.cipher.clone(),
        starts_at: item.starts_at.to_string(),
        label: item.label.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_digest_views(items: &[RolloutDigest]) -> Vec<RolloutDigestView> {
    let mut views: Vec<RolloutDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_digest_view(item));
    }
    views
}

pub fn sum_rollout_digest_view(view: &RolloutDigestView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_starts_at as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_digest("code-801", "name-818");
        let view = to_rollout_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
