use super::*;

use acme_kit::text;

/// How screens and exports show a primary failover digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverDigestView {
    pub code_text: String,
    pub title: String,
    pub amount_text: String,
    pub attempts_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_primary_failover_digest_view(item: &PrimaryFailoverDigest) -> PrimaryFailoverDigestView {
    PrimaryFailoverDigestView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        amount_text: format!("{:.2}", item.amount),
        attempts_text: item.attempts.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_failover_digest_views(items: &[PrimaryFailoverDigest]) -> Vec<PrimaryFailoverDigestView> {
    let mut views: Vec<PrimaryFailoverDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_failover_digest_view(item));
    }
    views
}

pub fn sum_primary_failover_digest_view(view: &PrimaryFailoverDigestView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_amount + (parsed_attempts as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_failover_digest("code-623", "title-762");
        let view = to_primary_failover_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_failover_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
