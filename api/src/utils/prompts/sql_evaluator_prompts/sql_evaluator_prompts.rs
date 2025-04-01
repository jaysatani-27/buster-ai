use serde_json::{json, Value};

pub fn sql_evaluation_system_prompt() -> String {
    r#"## TASK

Your task is to evaluate the SQL copilot's response based on specific criteria and provide responses to each question in a JSON object, including a brief explanation for each answer.

Please consider the following criteria:

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


### GENERAL GUIDELINES
- The user has full secure access to all the data.
- If the user does not request a specific time period, it's possible the query may default to the last year."#.to_string()
}

pub fn sql_evaluation_user_prompt(
    input: &String,
    sql_copilot_response: &String,
    datasets: &String,
) -> String {
    format!(
        r#"## DATASET
{} 
        
##USER REQUEST
{}

##SQL COPILOT RESPONSE
{}"#,
        datasets, input, sql_copilot_response
    )
}

pub fn sql_evaluation_json_schema() -> Value {
    json!(
      {
         "name": "sql_review_schema",
         "schema": {
           "type": "object",
           "properties": {
             "answerable": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "boolean",
                   "description": "Indicates if the query is answerable."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "syntactic_correctness": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "boolean",
                   "description": "Indicates if the syntax of the query is correct."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "accuracy_of_sql_semantics": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "boolean",
                   "description": "Indicates if the SQL semantics are accurate."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "level_of_assumptions": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "string",
                   "enum": [
                     "straightforward with no assumptions",
                     "straightforward with minor assumptions",
                     "not straightforward and major assumptions were made"
                   ],
                   "description": "The level of assumptions present in the SQL query."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "avoidance_of_hallucinations": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "boolean",
                   "description": "Indicates if hallucinations were avoided."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "not_doing_extra": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "boolean",
                   "description": "Indicates if no extra actions were taken beyond the request."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "appropriate_use_of_available_data": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "boolean",
                   "description": "Indicates if available data was used appropriately."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             },
             "user_satisfaction": {
               "type": "object",
               "properties": {
                 "value": {
                   "type": "string",
                   "enum": [
                     "Not at all",
                     "Partially",
                     "Mostly",
                     "Completely"
                   ],
                   "description": "Level of user satisfaction with the output."
                 },
                 "explanation": {
                   "type": "string",
                   "description": "Your brief explanation here."
                 }
               },
               "required": [
                 "value",
                 "explanation"
               ],
               "additionalProperties": false
             }
           },
           "required": [
             "answerable",
             "syntactic_correctness",
             "accuracy_of_sql_semantics",
             "level_of_assumptions",
             "avoidance_of_hallucinations",
             "alignment_with_user_request",
             "not_doing_extra",
             "appropriate_use_of_available_data",
             "user_satisfaction"
           ],
           "additionalProperties": false
         }
       }
    )
}
