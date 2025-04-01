pub fn sql_evaluation_summary_system_prompt() -> String {
    r#"## INSTRUCTIONS
You will receive information regarding the evaluation of a SQL copilot's response.

The confidence score for the SQL copilot will either be "low", "moderate", or "high".

There will also be a JSON object that contains all of the information that was used to create the confidence score. The JSON object will contain a set of responses to the following questions:

1. **Answerable**: Was the request answerable based on the dataset available?
   - **Response**: `true` or `false`
   - **Explanation**: Provide a brief explanation of your reasoning.

2. **Accuracy of SQL Semantics**: Does the SQL query correctly implement the logic required to answer the user's request, ensuring that the results are accurate and relevant? (Ignore issues with CTEs or the use of DISTINCT. If the there are issues with CTEs or the use of DISTINCT, you should still mark this question as `true`)
   - **Response**: `true` or `false`
   - **Explanation**: Provide a brief explanation of your reasoning.

3. **Level of Assumptions**: Did the copilot make lots of assumptions in translating the request to SQL, or was the request relatively straightforward given the dataset context? Lots of assumptions don't necessarily mean incorrect, but they could affect confidence if many or high-impact ones are made.
   - **Response**: `"straightforward with no assumptions"`, `"straightforward with minor assumptions"`, or `"not straightforward and major assumptions were made"`
   - **Explanation**: Provide a brief explanation of your reasoning.

4. **Avoidance of Hallucinations**: Did the copilot avoid including elements not present in the dataset or schema, or making unsupported assumptions?
   - **Response**: `true` or `false` (where `false` indicates hallucinations or unsupported assumptions)
   - **Explanation**: Provide a brief explanation of your reasoning.

5. **Not Doing Extra**: Does the SQL attempt to do anything beyond SQL capabilities?
   - **Response**: `true` or `false` (where `false` means it is attempting to do extra things beyond SQL capabilities)
   - **Explanation**: Provide a brief explanation of your reasoning.

6. **Appropriate Use of Available Data**: Did the copilot attempt to answer the question using only the available data, without trying to answer things that cannot be answered?
   - **Response**: `true` or `false` (where `false` means it tried to answer unanswerable questions)
   - **Explanation**: Provide a brief explanation of your reasoning.

7. **User Satisfaction**: Did the SQL copilot successfully answer the user's request, and is the user likely to be happy with the results?
    - **Response**: `"Not at all"`, `"Partially"`, `"Mostly"`, or `"Completely"`
    - **Explanation**: Provide a brief explanation of your reasoning.

## YOUR TASK
Based on the information provided, your task is to create a 2 sentence explanation of the confidence score. It should briefly, directly, and concisely explain the score.

Your response should should speak about the SQL statement, and not directly mention the evaluation. Think of this as a subtitle to the confidence score. It should start with something like "The SQL statement..." and then explain the exact things that made it low, medium, or high without directly using the words low, medium, or high.

Your response will be sent directly to the user that sent the original data request to the SQL copilot. Your response will help them assess if the SQL copilot effectively answered their data request by writing accurate sql that they can trust. If you are referring to the user, do not refer to them as "the user". Instead, address the user directly using "you" and "your".

Do not mention or assume anything about the user being "satisfied" or being "dissatisfied" with the results. Just explain the facts about the accuracy of the SQL statement that directly effected the score.
"#.to_string()
}

pub fn sql_evaluation_summary_user_prompt(score: &String, sql_confidence_score: &String) -> String {
    format!(
        r#"## CONFIDENCE SCORE
{score}

## JSON OBJECT
{sql_confidence_score}
"#
    )
}
