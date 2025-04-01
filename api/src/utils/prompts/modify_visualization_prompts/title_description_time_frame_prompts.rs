pub fn title_description_time_frame_system_prompt() -> String {
    format!(
        r#"### TASK
You will be given a user request as well as the SQL statement and chart type that were returned to the user. Your task is split into three granular steps.

1. Generate a metric title
- Generate a descriptive metric title. 
- You should try to match terminology found in the users request/question. 
- Sometimes the SQL statement and visualization will not directly answer the user request. If this is the case, the metric title should be an accurate representation of what is shown in the visualization (not the user request). You should also classify the user request and SQL statement/visualization as perfect_match (the SQL statement/visualization directly answer all aspects of the user request) or discrepencies_identified (the SQL statement/thought process do not directly answer all aspects of the user request).
- Do not mention the chart/visualization type in the metric title. If the user request contains specific charting or visualization requests, you should still not include this in the metric title and, instead, just focus on the visualization that was generated as a result of the SQL statement. 
- The title should contain no special characters.

2. Generate a metric summary question
- Generate a a simple 'summary question' for the user request/question and the SQL statement. This is essentially transforming the metric title into a natural language question. This will be displayed as the subtitle or description of the metric.
- You should try to match terminology found in the users request/question.
- Ignore charting and visualization requests in the summary question and focus on the data being requested.
- The question should contain no special characters.

3. Identify the time frame that is used in the SQL statement
-  There are two types of time periods: ['periodic_metric', 'rolling_metric']
- periodic_metric: The time period is determined by a fixed span of time (from one particular date to another). This means that the data displayed will never change because the dates are fixed.
- If the time period is a 'periodic_metric', your response should be formatted as: 'Jan 13, 2021 - Jan 18, 2022', 'Nov 1, 2023 - Nov 30, 2023', 'Jan 1, 2020 - Dec 30, 2020', 'Mar 29, 2011 - Jul 4, 2021', 'Comparison: Nov 1, 2024 - Nov 7, 2024 and Nov 8, 2024 - Nov 14, 2024', 'Comparison: 2000 and 2001', etc
- rolling_metric: The time period is determined by taking the present day as a starting point. This means that the data displayed will always correspond to the last 7, 15, 30 days (for example) and will change every day. If no time period is specified in the SQL statement, the metric will display it's all time data (this is still a rolling_metric).
- If the time period is a 'rolling_metric', your response should be formatted as: 'Today', 'Yesterday', 'Last 7 days', 'Last 30 days', 'Last 2 months', 'Last year', 'Last 2 years', 'All time', 'Comparison: Last 7 days and 7 days prior', 'Comparison: This month and Last month', 'Comparison: September of this year and October of last year', etc
- Your response should be the time period that the visualization/data will display. 
- Do not explain anything. Only respond with the time period.

## OUTPUT
Your output should be in JSON and follow this schema:

```json
{{
  "metric_title": {{
    "classification": perfect_match or discrepencies_identified,
    "output": "Metric title here..."
  }},
  "metric_summary_question": {{
    "output": "Metric summary question here..."
  }},
  "time_frame": {{
    "type": periodic_metric or rolling_metric,
    "output": "Time frame here..."
  }}
}}
```

Ensure the JSON is properly formatted so it can be parsed and used in code."#
    )
}

pub fn title_description_time_frame_user_prompt(
    prompt: &String,
    sql: &String,
    thoughts: &String,
) -> String {
    format!(
        r#"## USER REQUEST
{}

## THOUGHT PROCESS
{}

## SQL STATEMENT
{}"#,
        prompt, thoughts, sql
    )
}
