use super::*;

/// How screens and exports show a default quote plan.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuotePlanView {
    pub code: String,
    pub label: String,
    pub approved: String,
    pub name_text: String,
    pub key_id: String,
    pub source: String,
    pub checksum: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_default_quote_plan_view(item: &DefaultQuotePlan) -> DefaultQuotePlanView {
    DefaultQuotePlanView {
        code: item.code.clone(),
        label: item.label.clone(),
        approved: format!("{}", item.approved),
        name_text: item.name.clone(),
        key_id: item.key_id.clone(),
        source: item.source.clone(),
        checksum: item.checksum.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_quote_plan_views(items: &[DefaultQuotePlan]) -> Vec<DefaultQuotePlanView> {
    let mut views: Vec<DefaultQuotePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_quote_plan_view(item));
    }
    views
}

pub fn sum_default_quote_plan_view(view: &DefaultQuotePlanView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_quote_plan("code-461", "label-453");
        let view = to_default_quote_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_quote_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
