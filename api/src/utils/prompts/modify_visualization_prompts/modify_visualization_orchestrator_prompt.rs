use serde_json::{json, Value};

pub fn modify_visualization_system_prompt() -> String {
    String::from(
        r#"# YOUR TASK
You are an AI assistant that helps in configuring data visualizations based on user requests. Your task is to:

1. **Interpret the user's request**, the current configuration, the SQL statement, and the data metadata provided.

2. **Decide which actions need to be taken** to fulfill the user's request, selecting from the following six actions:
   - `format_columns`
   - `stylize_columns`
   - `configure_charts`
   - `stylize_global_settings`
   - `cannot_do_requested_action_response`

3. For each action you select, **provide a brief description (`data_analyst_ticket`)** specifying which parts of the user's request this action addresses. **Rewrite the user's request concisely, removing unnecessary details.**

4. If no previous chart configuration is provided, you should always include a `configure_charts` action that instructs how to build the specified chart. For example: "Build a line chart with the x axis set to 'date' and the y axis set to 'sales'."

5. **Output your answer in JSON format**, conforming to the `visualization_actions_selector` schema.

## ADDITIONAL CONSIDERATIONS
- If no previous chart configuration is provided, you should always include a `configure_charts` action.
- Chart type is required for the `configure_charts` action.
- For styling changes, first determine whether the request applies at a global (entire chart) level or a column/series-specific level:
  - **Global styling (`stylize_global_settings`)**: Use for chart-wide appearance or behavior changes. Examples include changing the color palette for the entire chart, adjusting the legend visibility, adding goal lines, changing grid lines, or modifying overall chart axes behavior.
  - **Column styling (`stylize_columns`)**: Use for adjustments to a single column/series within the chart. Examples include changing line thickness for a single series, enabling data labels for a specific column, sorting a single column, or making a single series display as a bar instead of a line.
- Chart orientation is in the `stylize_global_settings` action. Do not mention orientation in any other action.
- All requests related to **coloring** should be done in the `stylize_global_settings` action if they apply to the entire chart. If the user requests specific colors or styling for a particular column, that is considered column styling and should be done under `stylize_columns`.
- An area chart is just a line chart with the column styling set to area.
- A stacked chart is just a bar chart with the column styling set to stack.

## DESCRIPTIONS OF ACTIONS

- **`format_columns`**:  
  Use this action to modify how data is displayed at the column-level. This includes changing display names, formatting numbers, adding prefixes/suffixes, specifying date formats, etc.  
  **Examples**:  
  - Change a column’s display name from "col_1" to "Total Sales"  
  - Format a column as currency with two decimal places
  
- **`stylize_columns`**:  
  Use this action for column-specific appearance and sorting changes. Adjust the appearance of a single series or a subset of columns within the chart.  
  **Examples**:  
  - Change a single series line thickness 
  - Change a single series line style (smooth, step, normal)
  - Turn data labels on/off for one specific column  
  - Sort columns by values  
  - Adjust bar roundness for a single series in a combo chart
  - Make line chart display as area or not.
  - Highlight data points on a line chart
  This action cannot do the following:
  - Change any colors.
  
- **`configure_charts`**:  
  Use this action to select or change the chart type and configure how data maps to axes. This is where you set the chart type (line, bar, scatter, table, etc.) and configure axes (X, Y, category, tooltip).
  You **will always specify the axes for the chart** and (if applicable) the tooltip or category settings. 
  This action can only set axis, category, and tooltip settings. Nothing else.
  Do not add things to the tooltip unless a user asks for it.
  If the user asks for anything outside of the axis, you should reference the other tools.
  This action cannot do the following:
  - Change line styles
  - Change line thickness
  - Adjust bar roundness
  - Display a line as an area
  - Change orientation
  - Stack or group bar columns
  
- **`stylize_global_settings`**:  
  Use this action for global-level styling changes that affect the entire chart’s overall appearance or behavior.  
  **Examples**:  
  - Configure chart orientation (horizontal, vertical)
  - Change colors or palette.
  - Sorting
  - Type of grouping (e.g. stack, group, etc.)
  - Bar show titles at top
  - Pie chart settings (since no column specific styling is allowed on pie.)
  - Toggle the legend visibility for the entire chart  
  - Show/hide grid lines globally  
  - Add global goal lines or trendlines and modify anything about the goal lines or trendlines
  - Configure top-level axis properties (e.g., X/Y axis behavior that affects the entire chart, not just one column)
  This action cannot do the following:
  - Change line styles
  - Change line thickness
  - Adjust bar roundness
  - Display a line as an area

- **`cannot_do_requested_action_response`**:  
  Use this action when the user requests something you cannot perform. Provide a `data_analyst_ticket` explaining which parts of the user's request this action addresses and why it should be used.

## GENERAL INSTRUCTIONS
- Analyze the user's request to determine which actions are needed. Multiple actions may be used if necessary.
- If part of a user request can be addressed while another cannot, respond with multiple actions accordingly.
- Ensure that when styling is requested, you carefully distinguish between changes that affect the entire chart (`stylize_global_settings`) and those that affect only a specific column or series (`stylize_columns`).
- The `configure_charts` action should always describe the axes, and (if applicable) the tooltip or category settings.
- Use the `stylize_global_settings` action for color requests.

## OUTPUT
Your output should be in the following JSON format adhering to the `visualization_actions_selector` schema:

```json
{
  "actions": [
    {
      "name": "<action_name>",
      "data_analyst_ticket": "<data_analyst_ticket>",
      "chart_type": "<chart_type>"
    },
    ...
  ]
}
```"#,
    )
}

pub fn modify_visualization_user_prompt(
    data_metadata: String,
    chart_config_context: String,
    column_metadata: String,
    modify_visualization_parse: String,
    final_thought: String,
) -> String {
    let mut user_request = if chart_config_context.is_empty() {
        modify_visualization_parse
            + "\n\n Please configure charts since there is no previous chart configuration."
    } else {
        modify_visualization_parse
    };

    if !final_thought.is_empty() {
        user_request += &format!("\n\n{}", final_thought);
    }

    format!(
        r#"## DATA METADATA
{data_metadata}

## PREVIOUS CHART CONFIG
{chart_config_context}

## PREVIOUS COLUMN LABEL FORMATS
{column_metadata}

## USER REQUEST
{user_request}"#,
        data_metadata = data_metadata,
        chart_config_context = chart_config_context,
        column_metadata = column_metadata,
        user_request = user_request,
    )
}

pub fn modify_visualization_prompt_schema() -> Value {
    json!({
      "name": "visualization_actions_selector",
      "schema": {
        "actions": {
          "type": "array",
          "items": {
            "anyOf": [
              {
                "type": "object",
                "properties": {
                  "name": { "type": "string", "enum": ["format_columns"] },
                  "data_analyst_ticket": {
                    "type": "string",
                  }
                },
                "required": ["name", "data_analyst_ticket"]
              },
              {
                "type": "object",
                "properties": {
                  "name": { "type": "string", "enum": ["stylize_columns"] },
                  "data_analyst_ticket": {
                    "type": "string",
                  }
                },
                "required": ["name", "data_analyst_ticket"]
              },
              {
                "type": "object",
                "properties": {
                  "name": { "type": "string", "enum": ["configure_charts"] },
                  "data_analyst_ticket": {
                    "type": "string",
                  },
                  "chart_type": {
                    "type": "string",
                    "enum": ["line", "bar", "pie", "scatter", "combo", "metric", "table"],
                    "description": "Required chart type that best aligns with user request or data structure."
                  }
                },
                "required": ["name", "data_analyst_ticket", "chart_type"]
              },
              {
                "type": "object",
                "properties": {
                  "name": { "type": "string", "enum": ["stylize_global_settings"] },
                  "description": {
                    "type": "string",
                    "description": "Use this action to stylize the entire chart. This action can only stylize the entire chart. It cannot stylize individual columns or series. Use this for color requests."
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                  }
                },
                "required": ["name", "data_analyst_ticket"]
              },
              {
                "type": "object",
                "properties": {
                  "name": { "type": "string", "enum": ["cannot_do"] },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "Use when request cannot be performed. Explain which parts of request cannot be done and why. Do not generate user response."
                  }
                },
                "required": ["name", "data_analyst_ticket"]
              }
            ]
          }
        },
        "type": "object",
        "properties": {},
        "required": ["actions"]
      }
    })
}
