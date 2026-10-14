use super::*;

use acme_kit::text;

/// How screens and exports show a capped suspension override.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverrideView {
    pub code: String,
    pub name_text: String,
    pub score_text: String,
    pub audience: String,
    pub rate: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_capped_suspension_override_view(item: &CappedSuspensionOverride) -> CappedSuspensionOverrideView {
    CappedSuspensionOverrideView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        score_text: format!("{:.2}", item.score),
        audience: text::truncate(&item.audience, 64),
        rate: format!("{:.2}", item.rate),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_suspension_override_views(items: &[CappedSuspensionOverride]) -> Vec<CappedSuspensionOverrideView> {
    let mut views: Vec<CappedSuspensionOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_suspension_override_view(item));
    }
    views
}

pub fn sum_capped_suspension_override_view(view: &CappedSuspensionOverrideView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_score + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_suspension_override("code-548", "name-545");
        let view = to_capped_suspension_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_suspension_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
