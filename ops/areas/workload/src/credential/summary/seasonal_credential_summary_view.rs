use super::*;

/// How screens and exports show a seasonal credential summary.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryView {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub amount: String,
    pub ends_at_text: String,
    pub principal_text: String,
    pub batch_size_text: String,
    pub quota_units: String,
    pub cluster: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_seasonal_credential_summary_view(item: &SeasonalCredentialSummary) -> SeasonalCredentialSummaryView {
    SeasonalCredentialSummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        segment: item.segment.clone(),
        amount: format!("{:.2}", item.amount),
        ends_at_text: item.ends_at.to_string(),
        principal_text: item.principal.clone(),
        batch_size_text: item.batch_size.to_string(),
        quota_units: item.quota_units.to_string(),
        cluster: item.cluster.clone(),
        reviewer_text: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_credential_summary_views(items: &[SeasonalCredentialSummary]) -> Vec<SeasonalCredentialSummaryView> {
    let mut views: Vec<SeasonalCredentialSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_credential_summary_view(item));
    }
    views
}

pub fn sum_seasonal_credential_summary_view(view: &SeasonalCredentialSummaryView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_amount + (parsed_ends_at as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_credential_summary("code-781", "title-217");
        let view = to_seasonal_credential_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_credential_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
