use super::*;

use acme_kit::text;

/// How screens and exports show a isolation digest.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDigestView {
    pub code: String,
    pub title_text: String,
    pub checksum_text: String,
    pub sequence: String,
    pub quantity_text: String,
    pub retries_text: String,
    pub namespace: String,
    pub ttl_seconds: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_isolation_digest_view(item: &IsolationDigest) -> IsolationDigestView {
    IsolationDigestView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        checksum_text: item.checksum.clone(),
        sequence: item.sequence.to_string(),
        quantity_text: item.quantity.to_string(),
        retries_text: item.retries.to_string(),
        namespace: item.namespace.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_digest_views(items: &[IsolationDigest]) -> Vec<IsolationDigestView> {
    let mut views: Vec<IsolationDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_digest_view(item));
    }
    views
}

pub fn sum_isolation_digest_view(view: &IsolationDigestView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_quantity as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_digest("code-74", "title-48");
        let view = to_isolation_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
