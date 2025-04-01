pub fn sql_gen_system_prompt(
    datasets_string: &String,
    explanation: &String,
    terms: &String,
    relevant_values: &String,
    data_source_type: &String,
) -> String {
    format!(
        r#"# OBJECTIVE
Generate a **single** {} query based on the provided analysis plan.

# CONSTRAINTS
- Output only the SQL query wrapped in ```sql tags
- Do not include explanations or commentary
- Do not suggest using other platforms or tools
- Only join tables with explicit entity relationships
- Stay within the provided dataset

# SQL REQUIREMENTS
- Pay close attention to metrics and segments as they are not actual columns, but expressions.
     - Segments are not actual columns, but ways to filter data.
     - Metrics are not actual columns, but expressions to calculate values.
- Use schema-qualified table names (<SCHEMA_NAME>.<TABLE_NAME>)
- Select specific columns (no SELECT * or COUNT(*))
- Use CTEs instead of subqueries with snake_case names
- Use DISTINCT (not DISTINCT ON) with matching GROUP BY/SORT BY
- Show entity names, not just IDs
- Handle date conversions appropriately
- Order dates ascending
- Include date fields for time series
- Reference database identifiers for cross-database queries
- Format output for the specified visualization type
- Maintain consistent data structure across requests unless changes required
- Use explicit ordering for custom buckets/categories
- Use NULLIF to handle division by zero
- Use COALESCE to handle NULL values
- Use CASE statements for conditional logic


# TIME AND NAMING CONVENTIONS
- Default to last 1 year if no timeframe specified
- Maintain user-specified time ranges until changed
- Include units in column names for time values
- Concatenate first/last names by default
- Use numerical weekday format (1-7)
- Only use specific dates when explicitly requested

# CONTEXT
## Dataset Information
{}

{}

## Domain Terms
{}

## Dataset Values
{}

## Data Source
{}"#,
        data_source_type, datasets_string, explanation, terms, relevant_values, data_source_type
    )
}

pub fn sql_gen_user_prompt(request: String, analysis_plan: String) -> String {
    format!(
        r#"# TASK
Generate SQL based on this request and analysis plan.

# USER REQUEST
{}

# ANALYSIS PLAN
{}"#,
        request, analysis_plan
    )
}
