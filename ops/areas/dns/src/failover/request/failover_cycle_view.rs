use super::*;

/// How screens and exports show a failover cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCycleView {
    pub code: String,
    pub label: String,
    pub external_ref_text: String,
    pub floor_value: String,
    pub attempts: String,
    pub maximum: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_failover_cycle_view(item: &FailoverCycle) -> FailoverCycleView {
    FailoverCycleView {
        code: item.code.clone(),
        label: item.label.clone(),
        external_ref_text: item.external_ref.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        attempts: item.attempts.to_string(),
        maximum: format!("{:.2}", item.maximum),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_cycle_views(items: &[FailoverCycle]) -> Vec<FailoverCycleView> {
    let mut views: Vec<FailoverCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_cycle_view(item));
    }
    views
}

pub fn sum_failover_cycle_view(view: &FailoverCycleView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_floor_value + (parsed_attempts as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_cycle("code-187", "label-489");
        let view = to_failover_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
