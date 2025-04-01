export type TooltipFormatterParams = {
  marker: string;

  componentType: 'series';
  // Series type
  seriesType: string;
  // Series index in option.series
  seriesIndex: number;
  // Series name
  seriesName: string;
  // Data name, or category name
  name: string;
  // Data index in input data array
  dataIndex: number;
  // Original data as input
  data: Object;
  // Value of data. In most series it is the same as data.
  // But in some series it is some part of the data (e.g., in map, radar)
  value: Array<string | number>;
  // encoding info of coordinate system
  // Key: coord, like ('x' 'y' 'radius' 'angle')
  // value: Must be an array, not null/undefined. Contain dimension indices, like:
  // {
  //     x: [2] // values on dimension index 2 are mapped to x axis.
  //     y: [0] // values on dimension index 0 are mapped to y axis.
  // }
  encode: {
    x: [number];
    y: [number];
  };
  // dimension names list
  dimensionNames: Array<String>;
  // data dimension index, for example 0 or 1 or 2 ...
  // Only work in `radar` series.
  dimensionIndex: number;
  // Color of data
  color: string;
  // The percentage of current data item in the pie/funnel series
  percent: number;
  // The ancestors of current node in the sunburst series (including self)
  treePathInfo: Array<any>;
  // The ancestors of current node in the tree/treemap series (including self)
  treeAncestors: Array<any>;

  axisType: 'xAxis.time' | 'value';
};
