pub fn failed_to_fix_sql_system_prompt() -> String {
    "### YOUR TASK
You recently tried to run a SQL query and fix it.  You were unsuccessful and may have an error message and the failed SQL query.

You should acknowledge that the user made a request and that you made some decisions. One of the decisions you made was to select a dataset.  Talk briefly about why you made that decision.

You should talk about how you tried to fix the SQL query three times, and that you failed.  Talk about the error that you got.

At the end make sure to apologize to the user

PLEASE OUTPUT THE SQL QUERY IN THE FOLLOWING JSON FORMAT:
```json
{
  \"sql\": \"SELECT...\"
}
```

### GENERAL GUIDELINES
- Keep your response under 50 words
- Do not output the failed SQL query in your response
- Keep this response fairly non-technical
- escape columns, datasets, tables, errors, etc. with backticks.
- You must output the sql query in the format specified above.
".to_string()
}

pub fn failed_to_fix_sql_user_prompt(
    input: &String,
    action_decisions: &Option<String>,
    dataset_selection: &Option<String>,
    sql: &Option<String>,
    error: &Option<String>,
) -> String {
    let mut message = format!("## USER MESSAGE\n{}", input);

    if let Some(decisions) = action_decisions {
        message.push_str("\n\n## DECISIONS MADE WHEN THE USER REQUEST WAS FIRST RECEIVED\n");
        message.push_str(decisions);
    }

    if let Some(dataset) = dataset_selection {
        message.push_str("\n\n## DATASET SELECTION AND REASONING\n");
        message.push_str(dataset);
    }

    if let Some(sql) = sql {
        message.push_str("\n\n## SQL\n");
        message.push_str(sql);
    }

    if let Some(error) = error {
        message.push_str("\n\n## ERROR\n");
        message.push_str(error);
    }

    message.push_str("\n\nPlease output the SQL query in the format specified above. (```sql ... ```)");

    message
}
