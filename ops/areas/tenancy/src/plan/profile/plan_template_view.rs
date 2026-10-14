use super::*;

/// How screens and exports show a plan template.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanTemplateView {
    pub code: String,
    pub label: String,
    pub discount_text: String,
    pub version_text: String,
    pub ratio: String,
    pub channel: String,
    pub scope_text: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_plan_template_view(item: &PlanTemplate) -> PlanTemplateView {
    PlanTemplateView {
        code: item.code.clone(),
        label: item.label.clone(),
        discount_text: format!("{:.2}", item.discount),
        version_text: item.version.to_string(),
        ratio: format!("{:.2}", item.ratio),
        channel: item.channel.clone(),
        scope_text: item.scope.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_template_views(items: &[PlanTemplate]) -> Vec<PlanTemplateView> {
    let mut views: Vec<PlanTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_template_view(item));
    }
    views
}

pub fn sum_plan_template_view(view: &PlanTemplateView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_discount + (parsed_version as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_template("code-281", "label-104");
        let view = to_plan_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
