use super::*;

/// How screens and exports show a lenient redaction milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionMilestoneView {
    pub code_text: String,
    pub name: String,
    pub amount: String,
    pub version: String,
    pub limit_text: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_lenient_redaction_milestone_view(item: &LenientRedactionMilestone) -> LenientRedactionMilestoneView {
    LenientRedactionMilestoneView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        version: item.version.to_string(),
        limit_text: item.limit.to_string(),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_redaction_milestone_views(items: &[LenientRedactionMilestone]) -> Vec<LenientRedactionMilestoneView> {
    let mut views: Vec<LenientRedactionMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_redaction_milestone_view(item));
    }
    views
}

pub fn sum_lenient_redaction_milestone_view(view: &LenientRedactionMilestoneView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_amount + (parsed_version as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_redaction_milestone("code-79", "name-614");
        let view = to_lenient_redaction_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_redaction_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
