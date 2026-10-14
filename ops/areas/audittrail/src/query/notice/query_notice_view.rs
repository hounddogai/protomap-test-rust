use super::*;

/// How screens and exports show a query notice.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryNoticeView {
    pub code: String,
    pub name_text: String,
    pub budget_text: String,
    pub margin: String,
    pub batch_size_text: String,
    pub title: String,
    pub cost: String,
    pub visible_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_query_notice_view(item: &QueryNotice) -> QueryNoticeView {
    QueryNoticeView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        margin: format!("{:.2}", item.margin),
        batch_size_text: item.batch_size.to_string(),
        title: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        visible_text: format!("{}", item.visible),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_notice_views(items: &[QueryNotice]) -> Vec<QueryNoticeView> {
    let mut views: Vec<QueryNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_notice_view(item));
    }
    views
}

pub fn sum_query_notice_view(view: &QueryNoticeView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_budget + parsed_margin + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_query_notice("code-462", "name-515");
        let view = to_query_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_query_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
