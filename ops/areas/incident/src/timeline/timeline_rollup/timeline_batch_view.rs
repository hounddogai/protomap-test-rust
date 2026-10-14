use super::*;

use acme_kit::text;

/// How screens and exports show a timeline batch.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBatchView {
    pub code: String,
    pub title_text: String,
    pub score_text: String,
    pub country: String,
    pub channel_text: String,
    pub minimum: String,
    pub ceiling: String,
    pub name: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_timeline_batch_view(item: &TimelineBatch) -> TimelineBatchView {
    TimelineBatchView {
        code: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 40),
        score_text: format!("{:.2}", item.score),
        country: item.country.clone(),
        channel_text: item.channel.clone(),
        minimum: format!("{:.2}", item.minimum),
        ceiling: format!("{:.2}", item.ceiling),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_batch_views(items: &[TimelineBatch]) -> Vec<TimelineBatchView> {
    let mut views: Vec<TimelineBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_batch_view(item));
    }
    views
}

pub fn sum_timeline_batch_view(view: &TimelineBatchView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_score + parsed_minimum + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_batch("code-830", "title-696");
        let view = to_timeline_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
