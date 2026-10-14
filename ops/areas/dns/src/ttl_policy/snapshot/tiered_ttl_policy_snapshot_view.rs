use super::*;

use acme_kit::text;

/// How screens and exports show a tiered ttl policy snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicySnapshotView {
    pub code: String,
    pub title: String,
    pub description_text: String,
    pub batch_size: String,
    pub currency_text: String,
    pub score: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_tiered_ttl_policy_snapshot_view(item: &TieredTtlPolicySnapshot) -> TieredTtlPolicySnapshotView {
    TieredTtlPolicySnapshotView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        description_text: item.description.clone(),
        batch_size: item.batch_size.to_string(),
        currency_text: item.currency.clone(),
        score: format!("{:.2}", item.score),
        cipher_text: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_ttl_policy_snapshot_views(items: &[TieredTtlPolicySnapshot]) -> Vec<TieredTtlPolicySnapshotView> {
    let mut views: Vec<TieredTtlPolicySnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_ttl_policy_snapshot_view(item));
    }
    views
}

pub fn sum_tiered_ttl_policy_snapshot_view(view: &TieredTtlPolicySnapshotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_ttl_policy_snapshot("code-332", "title-87");
        let view = to_tiered_ttl_policy_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_ttl_policy_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
