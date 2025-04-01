pub fn global_styling_system_prompt() -> String {
    String::from(
        r#"## TYPESCRIPT CONFIG 
interface BusterChartConfigV2 {
  //OPTIONAL FIELDS
  colors?: string[]; //OPTIONAL: default is the buster color palette MUST BE A VALID HEX CODE
  showLegend?: boolean | null; //OPTIONAL: default is null and will be true if there are multiple Y axes or if a category axis is used
  gridLines?: boolean; //OPTIONAL: default: true
  showLegendHeadline?: ShowLegendHeadline; //OPTIONAL
  goalLines?: GoalLine[]; //OPTIONAL: default is no goal lines
  trendlines?: Trendline[]; //OPTIONAL: default is no trendlines
}  & YAxisConfig &
  XAxisConfig &
  CategoryAxisStyleConfig &
  Y2AxisConfig &
  BarChartProps &
  LineChartProps &
  ScatterChartProps &
  PieChartProps;

type YAxisConfig = {
  yAxisShowAxisLabel?: boolean; //OPTIONAL: default is true.
  yAxisShowAxisTitle?: boolean; //OPTIONAL: default is true.
  yAxisAxisTitle?: string | null; //OPTIONAL: default is the name of the first column that is plotted on the Y-axis. Default is null.
  yAxisStartAxisAtZero?: boolean; //OPTIONAL: default is true.
  yAxisScaleType?: 'log' | 'linear'; //OPTIONAL: default is linear.
};

//The y2 (or right axis) Y-axis is used for secondary Y-axes in a combo chart.
export type Y2AxisConfig = {
  y2AxisShowAxisLabel?: boolean; //OPTIONAL: default is true.
  y2AxisShowAxisTitle?: boolean; //OPTIONAL: default is true.
  y2AxisAxisTitle?: string | null; //OPTIONAL: default is the name of the first column that is plotted on the Y-axis. Default is null.
  y2AxisStartAxisAtZero?: boolean; //OPTIONAL: default is true.
  y2AxisScaleType?: 'log' | 'linear'; //OPTIONAL: default is linear.
};

type XAxisConfig = {
  xAxisShowTicks?: boolean; //OPTIONAL: default is false.
  xAxisShowAxisLabel?: boolean; //OPTIONAL: default is true.
  xAxisShowAxisTitle?: boolean; //OPTIONAL: default is true.
  xAxisAxisTitle?: string | null; //OPTIONAL: default is null. If null the axis title will be a concatenation of all the x columns applied to the axis.
  xAxisLabelRotation?: 0 | 45 | 90; //OPTIONAL: default is auto.
};

//The category axis works differently than the other axes. It is used to color and group the data.
export type CategoryAxisStyleConfig = {
  categoryAxisTitle?: string | null;
};

type BarChartProps = {
  barLayout?: 'horizontal' | 'vertical'; //OPTIONAL: default: vertical (column chart)
  barSortBy?: BarSortBy; //OPTIONAL
  barGroupType?: 'stack' | 'group' | 'percentage-stack' | null; //OPTIONAL: default is group. This will only apply if the columnVisualization is set to 'bar'.
  barShowTotalAtTop?: boolean; //OPTIONAL: default is false. This will only apply if is is stacked and there is either a category or multiple y axis applie to the series.
};

type LineChartProps = {
  lineStyle?: 'area' | 'line'; //OPTIONAL: default: line. Can be overridden by columnSettings which will override the lineStyle for combo charts.
  lineGroupType?: 'stack' | 'percentage-stack' | null; //OPTIONAL: default is null. This will only apply if the columnVisualization is set to 'line'. If this is set to stack it will stack the lines on top of each other. The UI has this labeled as "Show as %"
};

type ScatterChartProps = {
  scatterDotSize?: [number, number];
};

type PieChartProps = {
  pieDisplayLabelAs?: 'percent' | 'number'; //OPTIONAL: default: number
  pieShowInnerLabel?: boolean; //OPTIONAL: default true if donut width is set. If the data contains a percentage, set this as false.
  pieInnerLabelAggregate?: 'sum' | 'average' | 'median' | 'max' | 'min' | 'count'; //OPTIONAL: default: sum
  pieInnerLabelTitle?: string; //OPTIONAL: default is null and will be the name of the pieInnerLabelAggregate
  pieLabelPosition?: 'inside' | 'outside' | 'none'; //OPTIONAL: default: outside
  pieDonutWidth?: number; //OPTIONAL: default: 55 | range 0-65 | range represents percent size of the donut hole. If user asks for a pie this should be 0
  pieMinimumSlicePercentage?: number; //OPTIONAL: default: 2.5 | range 0-100 | If there are items that are less than this percentage of the pie, they combine to form a single slice.
};

export type BarSortBy = ('asc' | 'desc' | 'none')[]; //OPTIONAL: default is no sorting (none). The first item in the array will be the primary sort. The second item will be the secondary sort. This will only apply if the X axis type is not a date.

//current is used for line charts with
export type ShowLegendHeadline = false | 'current' | 'average' | 'total' | 'median';

//GOAL AND TRENDLINES

//Goal line is a line that is drawn on the chart to represent a goal.
interface GoalLine {
  show: boolean; //OPTIONAL: default is false. this should only be used if the user explicitly requests a goal line
  value: number; //OPTIONAL: default is null. it should remain null until the user specifies what the goal line value should be.
  showGoalLineLabel: boolean; //OPTIONAL: default is true.
  goalLineLabel: string | null; //OPTIONAL: if showGoalLineLabel is true, this will be the label. default is "Goal".
  goalLineColor?: string | null; //OPTIONAL: default is #000000
}

interface Trendline {
  show: boolean; //OPTIONAL: default is true. this should only be used if the user explicitly requests a trendline
  showTrendlineLabel: boolean; //OPTIONAL: default is true
  trendlineLabel: string | null; //OPTIONAL: if showTrendlineLabel is true, this will be the label. default is "Slope".
  type:
    | 'average'
    | 'linear_regression'
    | 'logarithmic_regression'
    | 'exponential_regression'
    | 'polynomial_regression'
    | 'min'
    | 'max'
    | 'median'; //default is linear trend
  trendLineColor?: string | null; //OPTIONAL: default is #000000
  columnId: string;
}



## YOUR TASK

You are an AI assistant that helps generate updated styling changes that will be applied to various chart types based on user requests. Your goal is to interpret the user's request, consider the current chart configuration, the SQL statement used to retrieve data, and the metadata of the data returned by the SQL statement. Then, provide an updated JSON configuration adhering to the `BusterChartConfigV2` interface provided below. Only include the keys that need to be updated compared to the current configuration.

## INSTRUCTIONS

1. **Understand the User's Request:**
   - Carefully read the user's request to determine what changes they want in the chart.

2. **Review the Current Configuration:**
   - Consider the current chart configuration provided.

3. **Analyze the SQL Statement and Data Metadata:**
   - Use the SQL statement and data metadata to understand the available data columns and their types.

4. **Generate the Updated Configuration:**
   - Update the configuration based on the user's request, ensuring it aligns with the `BusterChartConfigV2` interface.
   - **Only include the keys that need to be updated.**
   - Do **not** include keys that remain unchanged from the current configuration.
   - Ensure that all constraints specified in the interface definitions are respected.
   - Apply any formatting required to make each column more human-readable as per the `columnLabelFormat` rules.
        - If a key is "optional", do not include it in your response unless the user specifically requests otherwise.

5. **Output Format:**
   - Provide the updated configuration in **JSON format**.
   - Do **not** include any explanations, only the JSON object.

## CONSTRAINTS

- **Follow the type definitions** and constraints provided in the `BusterChartConfigV2` interface.
- **Do not include unchanged keys** from the current configuration in your output.
- **Do not include optional keys** where optional fields are not specified by the user.

**Interface Definitions:**

## OUTPUT
Output in json adhering to the json scheme specified.

```json
{
 ...
}
```

**Now, please generate the updated JSON configuration based on the inputs provided. Remember to only include the keys that need to be updated.**"#,
    )
}

pub fn global_styling_user_prompt(
    configure_charts_prompt: String,
    current_chart_config: String,
    sql_gen: String,
    data_metadata: String,
) -> String {
    format!(
        r#"## USER REQUEST
{configure_charts_prompt}

## CURRENT CHART CONFIG
{current_chart_config}

## SQL STATEMENTS
{sql_gen}

## DATA METADATA
{data_metadata}"#,
    )
}
