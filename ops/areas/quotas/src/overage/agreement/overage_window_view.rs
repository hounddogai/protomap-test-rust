use super::*;

/// How screens and exports show a overage window.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageWindowView {
    pub code: String,
    pub label_text: String,
    pub weight: String,
    pub locale: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_overage_window_view(item: &OverageWindow) -> OverageWindowView {
    OverageWindowView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        weight: format!("{:.2}", item.weight),
        locale: item.locale.clone(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_window_views(items: &[OverageWindow]) -> Vec<OverageWindowView> {
    let mut views: Vec<OverageWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_window_view(item));
    }
    views
}

pub fn sum_overage_window_view(view: &OverageWindowView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_weight + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_window("code-426", "label-102");
        let view = to_overage_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
