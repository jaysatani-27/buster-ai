use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ViewType {
    Chart,
    Table,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChartType {
    Line,
    Bar,
    Scatter,
    Pie,
    Metric,
    Table,
    Combo,
}

impl ChartType {
    pub fn to_string(&self) -> String {
        match self {
            ChartType::Line => "line".to_string(),
            ChartType::Bar => "bar".to_string(),
            ChartType::Scatter => "scatter".to_string(),
            ChartType::Pie => "pie".to_string(),
            ChartType::Metric => "metric".to_string(),
            ChartType::Table => "table".to_string(),
            ChartType::Combo => "combo".to_string(),
        }
    }

    pub fn from_string(chart_type: &str) -> ChartType {
        match chart_type {
            "line" => ChartType::Line,
            "bar" => ChartType::Bar,
            "scatter" => ChartType::Scatter,
            "pie" => ChartType::Pie,
            "metric" => ChartType::Metric,
            "table" => ChartType::Table,
            "combo" => ChartType::Combo,
            _ => ChartType::Table,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BusterChartConfig {
    pub selected_chart_type: ChartType,
    pub selected_view: ViewType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_label_formats: Option<HashMap<String, ColumnLabelFormat>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_settings: Option<HashMap<String, ColumnSettings>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_legend: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_lines: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_legend_headline: Option<ShowLegendHeadline>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_lines: Option<Vec<GoalLine>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trendlines: Option<Vec<Trendline>>,
    #[serde(flatten)]
    pub y_axis_config: YAxisConfig,
    #[serde(flatten)]
    pub x_axis_config: XAxisConfig,
    #[serde(flatten)]
    pub y2_axis_config: Y2AxisConfig,
    #[serde(flatten)]
    pub bar_chart_props: BarChartProps,
    #[serde(flatten)]
    pub line_chart_props: LineChartProps,
    #[serde(flatten)]
    pub scatter_chart_props: ScatterChartProps,
    #[serde(flatten)]
    pub pie_chart_props: PieChartProps,
    #[serde(flatten)]
    pub table_chart_props: TableChartProps,
    #[serde(flatten)]
    pub combo_chart_props: ComboChartProps,
    #[serde(flatten)]
    pub metric_chart_props: MetricChartProps,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LineChartProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_group_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BarChartProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_and_line_axis: Option<BarAndLineAxis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_sort_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_group_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_show_total_at_top: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScatterChartProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scatter_axis: Option<ScatterAxis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scatter_dot_size: Option<(f64, f64)>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PieChartProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_chart_axis: Option<PieChartAxis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_display_label_as: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_show_inner_label: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_inner_label_aggregate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_inner_label_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_label_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_donut_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_minimum_slice_percentage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TableChartProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_column_order: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_column_widths: Option<HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_header_background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_header_font_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_column_font_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ComboChartProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combo_chart_axis: Option<ComboChartAxis>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MetricChartProps {
    pub metric_column_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_value_aggregate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_header: Option<MetricTitle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_sub_header: Option<MetricTitle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_value_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricTitle {
    String(String),
    Derived(DerivedMetricTitle),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DerivedMetricTitle {
    pub column_id: String,
    pub use_value: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShowLegendHeadline {
    Bool(bool),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_data_labels: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_visualization: Option<String>,
    #[serde(flatten)]
    pub line_settings: LineColumnSettings,
    #[serde(flatten)]
    pub bar_settings: BarColumnSettings,
    #[serde(flatten)]
    pub dot_settings: DotColumnSettings,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LineColumnSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_dash_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_symbol_size: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BarColumnSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_roundness: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DotColumnSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_symbol_size: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnLabelFormat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_separator_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_fraction_digits: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_fraction_digits: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_missing_data_with: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_relative_time: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_utc: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make_label_human_readable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convert_number_to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoalLine {
    pub show: bool,
    pub value: f64,
    pub show_goal_line_label: bool,
    pub goal_line_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_line_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trendline {
    pub show: bool,
    pub show_trendline_label: bool,
    pub trendline_label: Option<String>,
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trendline_color: Option<String>,
    pub column_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct YAxisConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_show_axis_label: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_show_axis_title: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_axis_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_start_axis_at_zero: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_scale_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Y2AxisConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2_axis_show_axis_label: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2_axis_show_axis_title: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2_axis_axis_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2_axis_start_axis_at_zero: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2_axis_scale_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct XAxisConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_show_ticks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_show_axis_label: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_show_axis_title: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_axis_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_label_rotation: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_data_zoom: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CategoryAxisStyleConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_show_total_at_top: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_axis_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BarAndLineAxis {
    pub x: Vec<String>,
    pub y: Vec<String>,
    pub category: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScatterAxis {
    pub x: Vec<String>,
    pub y: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComboChartAxis {
    pub x: Vec<String>,
    pub y: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PieChartAxis {
    pub x: Vec<String>,
    pub y: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<Vec<String>>,
}
