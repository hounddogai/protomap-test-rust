use super::*;

use acme_kit::text;

/// How screens and exports show a regional sink template.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSinkTemplateView {
    pub code: String,
    pub label: String,
    pub amount: String,
    pub burst: String,
    pub sequence: String,
    pub threshold_text: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_regional_sink_template_view(item: &RegionalSinkTemplate) -> RegionalSinkTemplateView {
    RegionalSinkTemplateView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        amount: format!("{:.2}", item.amount),
        burst: item.burst.to_string(),
        sequence: item.sequence.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_sink_template_views(items: &[RegionalSinkTemplate]) -> Vec<RegionalSinkTemplateView> {
    let mut views: Vec<RegionalSinkTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_sink_template_view(item));
    }
    views
}

pub fn sum_regional_sink_template_view(view: &RegionalSinkTemplateView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_amount + (parsed_burst as f64) + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_sink_template("code-580", "label-348");
        let view = to_regional_sink_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_sink_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
