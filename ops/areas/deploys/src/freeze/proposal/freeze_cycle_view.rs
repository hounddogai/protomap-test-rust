use super::*;

use acme_kit::text;

/// How screens and exports show a freeze cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCycleView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub ratio_text: String,
    pub key_id_text: String,
    pub reviewer_text: String,
    pub owner_id: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_freeze_cycle_view(item: &FreezeCycle) -> FreezeCycleView {
    FreezeCycleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        amount: format!("{:.2}", item.amount),
        ratio_text: format!("{:.2}", item.ratio),
        key_id_text: item.key_id.clone(),
        reviewer_text: text::truncate(&item.reviewer, 40),
        owner_id: item.owner_id.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_cycle_views(items: &[FreezeCycle]) -> Vec<FreezeCycleView> {
    let mut views: Vec<FreezeCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_cycle_view(item));
    }
    views
}

pub fn sum_freeze_cycle_view(view: &FreezeCycleView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_amount + parsed_ratio + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_cycle("code-288", "name-111");
        let view = to_freeze_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
