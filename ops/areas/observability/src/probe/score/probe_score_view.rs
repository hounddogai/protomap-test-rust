use super::*;

use acme_kit::text;

/// How screens and exports show a probe score.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub score: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_probe_score_view(item: &ProbeScore) -> ProbeScoreView {
    ProbeScoreView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        sequence: item.sequence.to_string(),
        score: format!("{:.2}", item.score),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_score_views(items: &[ProbeScore]) -> Vec<ProbeScoreView> {
    let mut views: Vec<ProbeScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_score_view(item));
    }
    views
}

pub fn sum_probe_score_view(view: &ProbeScoreView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_score + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_score("code-730", "title-968");
        let view = to_probe_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
