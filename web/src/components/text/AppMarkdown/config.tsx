import { Components } from 'react-markdown';
import React from 'react';
import { BusterTimestampProps } from './AppMarkdown_BusterTimestamp';
import { BusterSuggestionProps } from './AppMarkdown_BusterSuggestion';
import { BusterDatasetsProps } from './AppMarkdown_BusterDatasets';

export enum AppMarkdownComponentType {
  BusterTimestamp = 'buster-timestamp',
  BusterSuggestion = 'buster-suggestion',
  BusterDatasets = 'buster-datasets'
}

//CUSTOM COMPONENTS
export interface CustomComponents extends Components {
  [AppMarkdownComponentType.BusterTimestamp]: React.FC<BusterTimestampProps>;
  [AppMarkdownComponentType.BusterSuggestion]: React.FC<BusterSuggestionProps>;
  [AppMarkdownComponentType.BusterDatasets]: React.FC<BusterDatasetsProps>;
}
