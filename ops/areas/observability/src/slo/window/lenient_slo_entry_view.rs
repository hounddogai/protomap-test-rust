use super::*;

/// How screens and exports show a lenient slo entry.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloEntryView {
    pub code_text: String,
    pub name_text: String,
    pub host_text: String,
    pub reviewer_text: String,
    pub source: String,
    pub label_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_lenient_slo_entry_view(item: &LenientSloEntry) -> LenientSloEntryView {
    LenientSloEntryView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        host_text: item.host.clone(),
        reviewer_text: item.reviewer.clone(),
        source: item.source.clone(),
        label_text: item.label.clone(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_slo_entry_views(items: &[LenientSloEntry]) -> Vec<LenientSloEntryView> {
    let mut views: Vec<LenientSloEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_slo_entry_view(item));
    }
    views
}

pub fn sum_lenient_slo_entry_view(view: &LenientSloEntryView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_slo_entry("code-422", "name-980");
        let view = to_lenient_slo_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_slo_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
