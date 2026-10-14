use super::*;

/// How screens and exports show a escalation group.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationGroupView {
    pub code_text: String,
    pub label: String,
    pub discount_text: String,
    pub amount_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_escalation_group_view(item: &EscalationGroup) -> EscalationGroupView {
    EscalationGroupView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        discount_text: format!("{:.2}", item.discount),
        amount_text: format!("{:.2}", item.amount),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_group_views(items: &[EscalationGroup]) -> Vec<EscalationGroupView> {
    let mut views: Vec<EscalationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_group_view(item));
    }
    views
}

pub fn sum_escalation_group_view(view: &EscalationGroupView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_discount + parsed_amount + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_group("code-25", "label-527");
        let view = to_escalation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
