use super::*;

/// How screens and exports show a lenient suspension profile.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionProfileView {
    pub code: String,
    pub label: String,
    pub margin: String,
    pub minimum: String,
    pub key_id: String,
    pub sequence: String,
    pub batch_size_text: String,
    pub approved: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_lenient_suspension_profile_view(item: &LenientSuspensionProfile) -> LenientSuspensionProfileView {
    LenientSuspensionProfileView {
        code: item.code.clone(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        minimum: format!("{:.2}", item.minimum),
        key_id: item.key_id.clone(),
        sequence: item.sequence.to_string(),
        batch_size_text: item.batch_size.to_string(),
        approved: format!("{}", item.approved),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_suspension_profile_views(items: &[LenientSuspensionProfile]) -> Vec<LenientSuspensionProfileView> {
    let mut views: Vec<LenientSuspensionProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_suspension_profile_view(item));
    }
    views
}

pub fn sum_lenient_suspension_profile_view(view: &LenientSuspensionProfileView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_margin + parsed_minimum + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_suspension_profile("code-984", "label-398");
        let view = to_lenient_suspension_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_suspension_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
