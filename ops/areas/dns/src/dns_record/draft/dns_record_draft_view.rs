use super::*;

/// How screens and exports show a dns record draft.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordDraftView {
    pub code: String,
    pub label_text: String,
    pub maximum: String,
    pub approved: String,
    pub margin_text: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_dns_record_draft_view(item: &DnsRecordDraft) -> DnsRecordDraftView {
    DnsRecordDraftView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        maximum: format!("{:.2}", item.maximum),
        approved: format!("{}", item.approved),
        margin_text: format!("{:.2}", item.margin),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_draft_views(items: &[DnsRecordDraft]) -> Vec<DnsRecordDraftView> {
    let mut views: Vec<DnsRecordDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_draft_view(item));
    }
    views
}

pub fn sum_dns_record_draft_view(view: &DnsRecordDraftView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_maximum + parsed_margin + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dns_record_draft("code-42", "label-811");
        let view = to_dns_record_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dns_record_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
