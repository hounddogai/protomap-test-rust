use super::*;

/// How screens and exports show a freeze window.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeWindowView {
    pub code: String,
    pub name_text: String,
    pub attempts_text: String,
    pub discount: String,
    pub threshold_text: String,
    pub retries_text: String,
    pub phone_text: String,
    pub owner_id_text: String,
    pub cost_text: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_freeze_window_view(item: &FreezeWindow) -> FreezeWindowView {
    FreezeWindowView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        attempts_text: item.attempts.to_string(),
        discount: format!("{:.2}", item.discount),
        threshold_text: format!("{:.2}", item.threshold),
        retries_text: item.retries.to_string(),
        phone_text: item.phone.clone(),
        owner_id_text: item.owner_id.clone(),
        cost_text: format!("{:.2}", item.cost),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_window_views(items: &[FreezeWindow]) -> Vec<FreezeWindowView> {
    let mut views: Vec<FreezeWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_window_view(item));
    }
    views
}

pub fn sum_freeze_window_view(view: &FreezeWindowView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_discount + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_window("code-121", "name-966");
        let view = to_freeze_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
