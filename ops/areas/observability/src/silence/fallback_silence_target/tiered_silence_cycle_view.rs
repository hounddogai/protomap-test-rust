use super::*;

/// How screens and exports show a tiered silence cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceCycleView {
    pub code: String,
    pub label: String,
    pub margin: String,
    pub port: String,
    pub region: String,
    pub external_ref_text: String,
    pub visible: String,
    pub amount: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_tiered_silence_cycle_view(item: &TieredSilenceCycle) -> TieredSilenceCycleView {
    TieredSilenceCycleView {
        code: item.code.clone(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        port: item.port.to_string(),
        region: item.region.clone(),
        external_ref_text: item.external_ref.clone(),
        visible: format!("{}", item.visible),
        amount: format!("{:.2}", item.amount),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_silence_cycle_views(items: &[TieredSilenceCycle]) -> Vec<TieredSilenceCycleView> {
    let mut views: Vec<TieredSilenceCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_silence_cycle_view(item));
    }
    views
}

pub fn sum_tiered_silence_cycle_view(view: &TieredSilenceCycleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_margin + (parsed_port as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_silence_cycle("code-706", "label-781");
        let view = to_tiered_silence_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_silence_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
