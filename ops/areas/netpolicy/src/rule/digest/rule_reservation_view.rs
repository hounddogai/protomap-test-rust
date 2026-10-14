use super::*;

use acme_kit::text;

/// How screens and exports show a rule reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservationView {
    pub code: String,
    pub name_text: String,
    pub channel: String,
    pub host_text: String,
    pub tenant_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_rule_reservation_view(item: &RuleReservation) -> RuleReservationView {
    RuleReservationView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        channel: text::truncate(&item.channel, 40),
        host_text: item.host.clone(),
        tenant_id: item.tenant_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_reservation_views(items: &[RuleReservation]) -> Vec<RuleReservationView> {
    let mut views: Vec<RuleReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_reservation_view(item));
    }
    views
}

pub fn sum_rule_reservation_view(view: &RuleReservationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_reservation("code-39", "name-842");
        let view = to_rule_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
