use super::*;

/// How screens and exports show a manual probe notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeNoticeView {
    pub code: String,
    pub name_text: String,
    pub cost: String,
    pub floor_value: String,
    pub amount: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_manual_probe_notice_view(item: &ManualProbeNotice) -> ManualProbeNoticeView {
    ManualProbeNoticeView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        floor_value: format!("{:.2}", item.floor_value),
        amount: format!("{:.2}", item.amount),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_probe_notice_views(items: &[ManualProbeNotice]) -> Vec<ManualProbeNoticeView> {
    let mut views: Vec<ManualProbeNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_probe_notice_view(item));
    }
    views
}

pub fn sum_manual_probe_notice_view(view: &ManualProbeNoticeView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_cost + parsed_floor_value + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_probe_notice("code-700", "name-255");
        let view = to_manual_probe_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_probe_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
