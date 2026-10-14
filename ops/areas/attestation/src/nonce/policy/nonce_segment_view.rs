use super::*;

use acme_kit::text;

/// How screens and exports show a nonce segment.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSegmentView {
    pub code_text: String,
    pub name: String,
    pub minimum: String,
    pub rotation_days: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_nonce_segment_view(item: &NonceSegment) -> NonceSegmentView {
    NonceSegmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        minimum: format!("{:.2}", item.minimum),
        rotation_days: item.rotation_days.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_segment_views(items: &[NonceSegment]) -> Vec<NonceSegmentView> {
    let mut views: Vec<NonceSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_segment_view(item));
    }
    views
}

pub fn sum_nonce_segment_view(view: &NonceSegmentView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_minimum + (parsed_rotation_days as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_nonce_segment("code-980", "name-986");
        let view = to_nonce_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_nonce_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
