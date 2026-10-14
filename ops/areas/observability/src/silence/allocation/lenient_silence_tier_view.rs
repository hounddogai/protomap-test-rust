use super::*;

/// How screens and exports show a lenient silence tier.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTierView {
    pub code_text: String,
    pub label: String,
    pub summary: String,
    pub amount: String,
    pub weight: String,
    pub capacity_text: String,
    pub issuer: String,
    pub maximum: String,
    pub rotation_days: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_lenient_silence_tier_view(item: &LenientSilenceTier) -> LenientSilenceTierView {
    LenientSilenceTierView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        summary: item.summary.clone(),
        amount: format!("{:.2}", item.amount),
        weight: format!("{:.2}", item.weight),
        capacity_text: item.capacity.to_string(),
        issuer: item.issuer.clone(),
        maximum: format!("{:.2}", item.maximum),
        rotation_days: item.rotation_days.to_string(),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_silence_tier_views(items: &[LenientSilenceTier]) -> Vec<LenientSilenceTierView> {
    let mut views: Vec<LenientSilenceTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_silence_tier_view(item));
    }
    views
}

pub fn sum_lenient_silence_tier_view(view: &LenientSilenceTierView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_amount + parsed_weight + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_silence_tier("code-966", "label-610");
        let view = to_lenient_silence_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_silence_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
