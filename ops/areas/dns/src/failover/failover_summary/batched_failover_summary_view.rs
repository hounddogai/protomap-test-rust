use super::*;

use acme_kit::text;

/// How screens and exports show a batched failover summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummaryView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub margin: String,
    pub principal: String,
    pub threshold: String,
    pub timezone_text: String,
    pub contact_email_text: String,
    pub fingerprint: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_batched_failover_summary_view(item: &BatchedFailoverSummary) -> BatchedFailoverSummaryView {
    BatchedFailoverSummaryView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        host: text::truncate(&item.host, 40),
        margin: format!("{:.2}", item.margin),
        principal: item.principal.clone(),
        threshold: format!("{:.2}", item.threshold),
        timezone_text: item.timezone.clone(),
        contact_email_text: item.contact_email.clone(),
        fingerprint: item.fingerprint.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_failover_summary_views(items: &[BatchedFailoverSummary]) -> Vec<BatchedFailoverSummaryView> {
    let mut views: Vec<BatchedFailoverSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_failover_summary_view(item));
    }
    views
}

pub fn sum_batched_failover_summary_view(view: &BatchedFailoverSummaryView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_margin + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_failover_summary("code-730", "title-394");
        let view = to_batched_failover_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_failover_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
