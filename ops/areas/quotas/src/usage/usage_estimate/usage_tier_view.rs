use super::*;

/// How screens and exports show a usage tier.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageTierView {
    pub code: String,
    pub label: String,
    pub duration_minutes: String,
    pub issuer: String,
    pub floor_value: String,
    pub ratio: String,
    pub threshold: String,
    pub contact_email: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_usage_tier_view(item: &UsageTier) -> UsageTierView {
    UsageTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        issuer: item.issuer.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        ratio: format!("{:.2}", item.ratio),
        threshold: format!("{:.2}", item.threshold),
        contact_email: item.contact_email.clone(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_tier_views(items: &[UsageTier]) -> Vec<UsageTierView> {
    let mut views: Vec<UsageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_tier_view(item));
    }
    views
}

pub fn sum_usage_tier_view(view: &UsageTierView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_floor_value + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_tier("code-821", "label-623");
        let view = to_usage_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
