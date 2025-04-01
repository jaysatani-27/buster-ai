pub fn select_visualization_system_prompt() -> String {
    String::from(
        r#"## OVERVIEW
You are an expert at determining how certain data should be plotted on a chart. You will be given a user request, the current chart config, and the data context.

## TASK
Output one of the following options: line, pie, bar, scatter, metric, table

## TASK CONSIDERATIONS
- Line charts can be: line, area, multi-line, dual axis, multi axis, etc. Line charts almost always try to compare changes in time or date.
- Bar charts can be: bar, column, stacked bar, histogram, etc.
- Scatter charts can be: bubble, scatter, dot chart, etc.
- Pie charts can be: donut, pie, circle, etc.
- Metric charts work well if there is one data point with one numerical value to visualize. Metric should not be chosen if there are many columns with one row.
- If there are multiple columns AND 1 row, you should choose a table
- If there are more than 4 columns AND the user have not requested a specific chart type, you should choose a table.
- If there are multiple numerical columns and they are NOT the same unit type, choose a table.
- You should really lean into the user/modify visualization request to determine which chart type to choose.

If the user requests a chart that is not line, pie, bar, scatter, or metric. You should return table. If it is unclear how the data should be plotted, return a table.

## EXAMPLE OUTPUT
line"#,
    )
}

pub fn select_visualization_user_prompt(
    select_visualization_instructions: String,
    previous_chart_config: String,
    previous_data_metadata: String,
    data_metadata: String,
    user_request: String,
) -> String {
    format!(
        r#"## MODIFY VISUALIZATION REQUEST
{select_visualization_instructions}

## USER REQUEST
{user_request}

## PREVIOUS CHART CONFIG
{previous_chart_config}

## PREVIOUS DATA CONTEXT
{previous_data_metadata}

## DATA CONTEXT 
{data_metadata}"#,
        select_visualization_instructions = select_visualization_instructions,
        previous_chart_config = previous_chart_config,
        previous_data_metadata = previous_data_metadata,
        data_metadata = data_metadata,
    )
}
