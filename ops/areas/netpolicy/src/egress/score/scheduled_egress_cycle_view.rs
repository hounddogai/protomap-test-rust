use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled egress cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressCycleView {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub fingerprint: String,
    pub visible: String,
    pub key_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_scheduled_egress_cycle_view(item: &ScheduledEgressCycle) -> ScheduledEgressCycleView {
    ScheduledEgressCycleView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        locale: text::truncate(&item.locale, 80),
        fingerprint: item.fingerprint.clone(),
        visible: format!("{}", item.visible),
        key_id: item.key_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_egress_cycle_views(items: &[ScheduledEgressCycle]) -> Vec<ScheduledEgressCycleView> {
    let mut views: Vec<ScheduledEgressCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_egress_cycle_view(item));
    }
    views
}

pub fn sum_scheduled_egress_cycle_view(view: &ScheduledEgressCycleView) -> Result<f64, String> {
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
        let item = new_scheduled_egress_cycle("code-1", "name-899");
        let view = to_scheduled_egress_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_egress_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
