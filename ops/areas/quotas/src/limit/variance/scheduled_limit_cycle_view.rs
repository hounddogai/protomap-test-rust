use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled limit cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycleView {
    pub code: String,
    pub name_text: String,
    pub amount: String,
    pub version_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_scheduled_limit_cycle_view(item: &ScheduledLimitCycle) -> ScheduledLimitCycleView {
    ScheduledLimitCycleView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        amount: format!("{:.2}", item.amount),
        version_text: item.version.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_limit_cycle_views(items: &[ScheduledLimitCycle]) -> Vec<ScheduledLimitCycleView> {
    let mut views: Vec<ScheduledLimitCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_limit_cycle_view(item));
    }
    views
}

pub fn sum_scheduled_limit_cycle_view(view: &ScheduledLimitCycleView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_amount + (parsed_version as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_limit_cycle("code-955", "name-836");
        let view = to_scheduled_limit_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_limit_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
