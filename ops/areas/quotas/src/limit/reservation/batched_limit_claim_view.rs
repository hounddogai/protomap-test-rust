use super::*;

/// How screens and exports show a batched limit claim.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimView {
    pub code: String,
    pub label: String,
    pub created_at: String,
    pub maximum_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_batched_limit_claim_view(item: &BatchedLimitClaim) -> BatchedLimitClaimView {
    BatchedLimitClaimView {
        code: item.code.clone(),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_limit_claim_views(items: &[BatchedLimitClaim]) -> Vec<BatchedLimitClaimView> {
    let mut views: Vec<BatchedLimitClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_limit_claim_view(item));
    }
    views
}

pub fn sum_batched_limit_claim_view(view: &BatchedLimitClaimView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_maximum + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_limit_claim("code-700", "label-598");
        let view = to_batched_limit_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_limit_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
