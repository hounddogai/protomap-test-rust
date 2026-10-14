use super::*;

use acme_kit::text;

/// How screens and exports show a primary postmortem target.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemTargetView {
    pub code_text: String,
    pub name: String,
    pub amount: String,
    pub summary_text: String,
    pub segment: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_primary_postmortem_target_view(item: &PrimaryPostmortemTarget) -> PrimaryPostmortemTargetView {
    PrimaryPostmortemTargetView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 40),
        amount: format!("{:.2}", item.amount),
        summary_text: item.summary.clone(),
        segment: item.segment.clone(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_postmortem_target_views(items: &[PrimaryPostmortemTarget]) -> Vec<PrimaryPostmortemTargetView> {
    let mut views: Vec<PrimaryPostmortemTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_postmortem_target_view(item));
    }
    views
}

pub fn sum_primary_postmortem_target_view(view: &PrimaryPostmortemTargetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_amount + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_postmortem_target("code-743", "name-808");
        let view = to_primary_postmortem_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_postmortem_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
