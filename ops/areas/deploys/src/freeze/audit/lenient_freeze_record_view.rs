use super::*;

use acme_kit::text;

/// How screens and exports show a lenient freeze record.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeRecordView {
    pub code: String,
    pub name: String,
    pub category: String,
    pub checksum_text: String,
    pub minimum_text: String,
    pub sequence: String,
    pub cluster: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_lenient_freeze_record_view(item: &LenientFreezeRecord) -> LenientFreezeRecordView {
    LenientFreezeRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        category: item.category.clone(),
        checksum_text: item.checksum.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        sequence: item.sequence.to_string(),
        cluster: item.cluster.clone(),
        timezone: text::truncate(&item.timezone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_freeze_record_views(items: &[LenientFreezeRecord]) -> Vec<LenientFreezeRecordView> {
    let mut views: Vec<LenientFreezeRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_freeze_record_view(item));
    }
    views
}

pub fn sum_lenient_freeze_record_view(view: &LenientFreezeRecordView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_minimum + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_freeze_record("code-346", "name-774");
        let view = to_lenient_freeze_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_freeze_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
