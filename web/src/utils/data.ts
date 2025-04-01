import { unparse } from 'papaparse';

export const convertJsonToCSV = (jsonArray: any[]) => {
  var csv = unparse(jsonArray);
  return csv;
};
