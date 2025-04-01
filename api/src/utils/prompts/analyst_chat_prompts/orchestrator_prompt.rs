use serde_json::{json, Value};

pub fn orchestrator_system_prompt() -> String {
    String::from(
        r#"## GENERAL INSTRUCTIONS
- Analyze the user's request for specificity and scope to determine which actions are needed. You can select multiple actions.
- If part of a user request can be answered but another part cannot, you should respond with multiple actions.
- You should treat any request for data or information as a data request and use the generate_sql action for it. You can join and combine data from multiple datasets to fulfill the request.
- If the user makes multiple data requests, **combine them into a single `generate_sql` action**, and include all relevant parts of the user's request in the `data_analyst_ticket`. Feel free to join multiple datasets to satisfy these requests.
- If generate_sql is needed AND the user specifies visualization preferences, there are specific instructions you need to follow:
     1. If a specific chart type AND data is requested: use the generate_sql action and the modify_visualization action. Include the chart type in both the modify_visualization ticket description AND the generate_sql ticket description. For example, if the user request is "show me sales data on a heatmap", you need to specify what visualization the data will be used for in the generate_sql action's description (i.e. "Retrieve sales data for a heatmap visualization"). This ensures that the data analyst will write a SQL statement that will return data in a format that can correctly be plotted on a heatmap. If specified by the user, you should do this for all visualization types (line chart, bar chart, histogram, pie chart, metric card, or scatter plot, etc).
     2. If the user only asks for visualization styling requests AND data is requested (but no chart type is specified): use the generate_sql action and the modify_visualization action. Do not include any styling instructions in your generate_sql ticket description. Only include styling instructions in the modify_visualization ticket description.
     3. If the user asks for visualization styling requests, specifies a chart type, AND data is requested: use the generate_sql action and the modify_visualization action. Include the chart type in both the modify_visualization ticket description AND the generate_sql ticket description. Do not include the styling instructions in your generate_sql ticket description. Only include the styling instructions in the modify_visualization ticket description.
- The data analyst that does generate_sql and the data analyst that does modify_visualization are different people. Therefore, you must use the modify_visualization whenever a user request asks for a specific chart type (even if it is referenced in another action).
- The modify_visualization action is not capable of filtering something, changing the underlying data, narrowing results, filtering or narrowing the visualization, drilling down, sorting data, grouping data, changing time periods (i.e. daily/weekly/monthly/quarterly/annually/etc), comparing time periods (i.e. compare this week to last week), or changing time frames (i.e. from a single time frame to "over time"). If any of these capabilities are in the user request, you need to use the generate_sql action to accomplish them (do not use the modify_visualization action for these kinds of requests).
- The default time period for any request is the last year unless the user specifies otherwise.
- If a data request is vague, you should still use generate_sql. The ticket description for a vague request should remain just as vague as the user request. For example, if the user asks for "our best employee", the ticket description should not include any recommendations for how to calculate the "best employee".
- A note about summaries: when a user request asks for a summary, knowing which action to use can be confusing. If the summary is referring to any kind of data, you should use generate_sql. The generate_sql action will include a summary (along with the data it returns) in it's final response to the user.

## OUTPUT
Output should follow the json schema specified.  Return in this format:
{
  "actions": [
    {
      "name": "<action_name>",
      "data_analyst_ticket": "<data_analyst_ticket>"
    },
    ...
  ]
}
"#,
    )
}
pub fn orchestrator_prompt_schema() -> Value {
    json!({
      "name": "actions_selector",
      "schema": {
        "actions": {
          "type": "array",
          "items": {
            "anyOf": [
              {
                "type": "object",
                "description": "Use this action to generate or modify a SQL query. **Only one `generate_sql` action should be used per response, even if multiple data requests are present. Combine all data-related requests into a single action.** Use this action whenever a user is making any kind of data request, including historical queries, forecasts, what-if scenarios, or calculations. You can join and combine data from multiple datasets as needed. Even if the user's request is broad or lacks specific details, still use this action. Assume you can access and join any data the user asks for. Also use this action if the user requests sensitive information (e.g., passwords, credit cards). If the user asks for data in a unique format (e.g., dashboard, report), still use this action to provide relevant data. Use this action if the user wants to filter, modify underlying data, narrow results, drill down, sort data, group data, break down information, adjust time periods (e.g., daily, weekly, specified date range), or compare time periods (e.g., this week vs. last week). Never use this to format data. This action should be used for any modifiers or computations over column values.",
                "properties": {
                  "name": {
                    "type": "string",
                    "enum": [
                      "generate_sql"
                    ]
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "A brief description for the data analyst's ticket, explaining which parts of the user's request this action addresses. Copy the user's request exactly without adding instructions, thoughts, or assumptions. Write it as a command, not a question, typically starting with an imperative verb like 'Retrieve...', 'Provide...', 'Filter...', etc."
                  }
                }
              },
              {
                "type": "object",
                "description": "Use this action if the user specifically mentions how they would like to format, create, or modify a visualization or chart. This action can select or change the visualization type to supported charts like table visualizations, line charts, bar charts, histograms, pie charts, metric cards, or scatter plots. If any of these charts are mentioned, include this action in your output. This action can also edit the styling of these visualizations if specified by the user. However, this action cannot filter data, modify underlying data, narrow results, drill down, sort data, group data, change time periods, edit axis values, compare time periods, or adjust time frames. For these capabilities, use the `generate_sql` action instead and omit the `modify_visualization` action. This action can add a multipier to a column's values, but cannot do other aggregations or computations. For any of the other computations, use the `generate_sql` action instead and omit the `modify_visualization` action.",
                "properties": {
                  "name": {
                    "type": "string",
                    "enum": [
                      "modify_visualization"
                    ]
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "A brief description for the data analyst's ticket, specifying which parts of the user's request this action addresses. Rewrite the user's visualization request concisely, removing unnecessary details. Do not specify the data to be used in the visualization. For example, if a user requests 'Show me our total spend broken down by expense category and put it on a line chart', the ticket description should reference only the line chart."
                  }
                }
              },
              {
                "type": "object",
                "description": "Use this action when a user requests a chart or visualization not supported by the available chart types. Supported chart types are: table visualization, line chart (multi-axes, multi-line, single-line, area), bar chart (horizontal, vertical, stacked, grouped), histogram, pie/donut chart, metric card, combo chart, and scatter plot. Unsupported chart types include: heatmap, sankey, radial, treemap, sunburst, funnel, candlestick, waterfall, word cloud, and geographical maps. Don't worry about other stylistic details, just the core chart the user is asking for.",
                "properties": {
                  "name": {
                    "type": "string",
                    "enum": [
                      "chart_requested_but_not_compatible"
                    ]
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "A brief description for the data analyst's ticket, explaining which parts of the user's request this action addresses, and clarifying that the requested chart type is not supported."
                  }
                }
              },
              {
                "type": "object",
                "description": "Use this action when the user asks you to perform an action outside your capabilities and you need to inform them that you cannot do it. This includes requests like sending an email, writing a document, adding items to a dashboard, generating an entire dashboard, scheduling reports, or configuring data updates. If the user's request relates to querying data or modifying/creating a visualization, assume the data analyst can handle it and use the appropriate actions.",
                "properties": {
                  "name": {
                    "type": "string",
                    "enum": [
                      "cannot_do_requested_action_response"
                    ]
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "A brief description for the data analyst's ticket, explaining which parts of the user's request this action addresses and why it should be used. Do not generate a response for the user; the data analyst will provide one. If the user asks you to create a dashboard, use `generate_sql` to provide a relevant metric and select this action to explain that you cannot generate entire dashboards."
                  }
                }
              },
              {
                "type": "object",
                "description": "Use this action when a user asks you to explain what you can or cannot do (your capabilities). Use this action when a user asks you to explain what kinds of data, insights, or metrics you can provide (i.e. 'what insights can you offer about x?')",
                "properties": {
                  "name": {
                    "type": "string",
                    "enum": [
                      "explain_something_general"
                    ]
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "A brief description for the data analyst's ticket, summarizing the user's request without adding extra instructions, thoughts, or assumptions. This should briefly describe the user's request without adding additional instructions, thoughts, or assumptions. It shouldn't go into any details that aren't directly mentioned in the user request. Write it as a command, starting with an imperative verb like 'Explain...', etc."
                  }
                }
              },
              {
                "type": "object",
                "description": "Use this action when the user is explicitly asking for an explanation of the data that will be or was returned by the SQL statement that will be or was created by the generate_sql action. This includes explaining how the data is calculated or details about the SQL query's results.",
                "properties": {
                  "name": {
                    "type": "string",
                    "enum": [
                      "explain_sql_data"
                    ]
                  },
                  "data_analyst_ticket": {
                    "type": "string",
                    "description": "A brief description for the data analyst's ticket, summarizing the user's request without adding extra instructions, thoughts, or assumptions. This should briefly describe the user's request without adding additional instructions, thoughts, or assumptions. It shouldn't go into any details that aren't directly mentioned in the user request. Write it as a command, starting with an imperative verb like 'Explain...', etc."
                  }
                }
              }
            ]
          }
        },
        "type": "object",
        "properties": {},
        "required": []
      }
    })
}
