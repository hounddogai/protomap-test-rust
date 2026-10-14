use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem score.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemScoreView {
    pub code: String,
    pub title: String,
    pub batch_size: String,
    pub rate_text: String,
    pub tenant_id: String,
    pub owner_id_text: String,
    pub name: String,
    pub minimum_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_postmortem_score_view(item: &PostmortemScore) -> PostmortemScoreView {
    PostmortemScoreView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        batch_size: item.batch_size.to_string(),
        rate_text: format!("{:.2}", item.rate),
        tenant_id: item.tenant_id.clone(),
        owner_id_text: item.owner_id.clone(),
        name: item.name.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_score_views(items: &[PostmortemScore]) -> Vec<PostmortemScoreView> {
    let mut views: Vec<PostmortemScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_score_view(item));
    }
    views
}

pub fn sum_postmortem_score_view(view: &PostmortemScoreView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_rate + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_score("code-553", "title-84");
        let view = to_postmortem_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
