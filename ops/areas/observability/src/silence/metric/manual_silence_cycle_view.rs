use super::*;

/// How screens and exports show a manual silence cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceCycleView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub version: String,
    pub minimum: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_manual_silence_cycle_view(item: &ManualSilenceCycle) -> ManualSilenceCycleView {
    ManualSilenceCycleView {
        code: item.code.clone(),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        version: item.version.to_string(),
        minimum: format!("{:.2}", item.minimum),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_silence_cycle_views(items: &[ManualSilenceCycle]) -> Vec<ManualSilenceCycleView> {
    let mut views: Vec<ManualSilenceCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_silence_cycle_view(item));
    }
    views
}

pub fn sum_manual_silence_cycle_view(view: &ManualSilenceCycleView) -> Result<f64, String> {
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
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_amount + (parsed_version as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_silence_cycle("code-233", "name-706");
        let view = to_manual_silence_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_silence_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
