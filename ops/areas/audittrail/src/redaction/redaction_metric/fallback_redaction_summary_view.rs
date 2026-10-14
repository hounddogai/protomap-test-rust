use super::*;

use acme_kit::text;

/// How screens and exports show a fallback redaction summary.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionSummaryView {
    pub code: String,
    pub title_text: String,
    pub ceiling: String,
    pub burst: String,
    pub timezone: String,
    pub cost: String,
    pub reviewer: String,
    pub owner_id_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_fallback_redaction_summary_view(item: &FallbackRedactionSummary) -> FallbackRedactionSummaryView {
    FallbackRedactionSummaryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        burst: item.burst.to_string(),
        timezone: text::truncate(&item.timezone, 120),
        cost: format!("{:.2}", item.cost),
        reviewer: item.reviewer.clone(),
        owner_id_text: item.owner_id.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_redaction_summary_views(items: &[FallbackRedactionSummary]) -> Vec<FallbackRedactionSummaryView> {
    let mut views: Vec<FallbackRedactionSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_redaction_summary_view(item));
    }
    views
}

pub fn sum_fallback_redaction_summary_view(view: &FallbackRedactionSummaryView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ceiling + (parsed_burst as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_redaction_summary("code-776", "title-330");
        let view = to_fallback_redaction_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_redaction_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
