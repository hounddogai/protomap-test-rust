use super::*;

use acme_kit::text;

/// How screens and exports show a failover plan.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlanView {
    pub code: String,
    pub name: String,
    pub discount: String,
    pub contact_email_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_failover_plan_view(item: &FailoverPlan) -> FailoverPlanView {
    FailoverPlanView {
        code: item.code.clone(),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        contact_email_text: text::truncate(&item.contact_email, 64),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_plan_views(items: &[FailoverPlan]) -> Vec<FailoverPlanView> {
    let mut views: Vec<FailoverPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_plan_view(item));
    }
    views
}

pub fn sum_failover_plan_view(view: &FailoverPlanView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_discount + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_plan("code-204", "name-629");
        let view = to_failover_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
