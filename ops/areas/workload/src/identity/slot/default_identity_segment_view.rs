use super::*;

/// How screens and exports show a default identity segment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentitySegmentView {
    pub code: String,
    pub name_text: String,
    pub rate: String,
    pub source: String,
    pub amount: String,
    pub updated_at_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_default_identity_segment_view(item: &DefaultIdentitySegment) -> DefaultIdentitySegmentView {
    DefaultIdentitySegmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        rate: format!("{:.2}", item.rate),
        source: item.source.clone(),
        amount: format!("{:.2}", item.amount),
        updated_at_text: item.updated_at.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_identity_segment_views(items: &[DefaultIdentitySegment]) -> Vec<DefaultIdentitySegmentView> {
    let mut views: Vec<DefaultIdentitySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_identity_segment_view(item));
    }
    views
}

pub fn sum_default_identity_segment_view(view: &DefaultIdentitySegmentView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_rate + parsed_amount + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_identity_segment("code-844", "name-787");
        let view = to_default_identity_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_identity_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
