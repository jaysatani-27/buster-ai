import React, { useContext } from 'react';
import { Text } from '@/components/text';
import { createStyles } from 'antd-style';
import { AppTooltip } from '@/components/tooltip';
import { AppMaterialIcons } from '@/components/icons';
import { useBusterNewThreadsContextSelector } from '@/context/Threads';

export interface BusterDatasetsProps {
  //we need to use this because of the way markdown parses :(
  'data-sets': string; //json {id: string, title: string, description: string}[]
}

const useStyles = createStyles(({ token, css }) => {
  return {
    item: css`
      border: 1px solid ${token.colorBorder};
      padding: ${token.paddingSM} ${token.paddingMD};
      border-radius: 4px;
      background-color: ${token.colorBgContainer};
      padding: 4px 8px;

      &:hover {
        background-color: ${token.controlItemBgHover};
      }
    `,
    icon: css`
      color: ${token.colorIcon};
    `
  };
});

export const BusterDatasets: React.FC<BusterDatasetsProps> = (rest) => {
  const datasetsParsed = parseDatasets(rest['data-sets']);
  const { styles, cx } = useStyles();

  const handleClick = (dataset: { id: string; title: string; description: string }) => {
    alert(dataset.id);
  };

  return (
    <div className="buster-datasets flex flex-wrap gap-2">
      {datasetsParsed.map((dataset) => (
        <div
          onClick={() => handleClick(dataset)}
          key={dataset.id}
          className={cx(
            styles.item,
            'cursor-pointer',
            'flex items-center gap-1 rounded-full border'
          )}>
          {dataset.description && (
            <AppTooltip title={dataset.description}>
              <AppMaterialIcons icon={'info'} className={cx(styles.icon)} />
            </AppTooltip>
          )}
          <Text>{dataset.title}</Text>
        </div>
      ))}
    </div>
  );
};

const parseDatasets = (datasets: string): { id: string; title: string; description: string }[] => {
  try {
    return JSON.parse(datasets) as { id: string; title: string; description: string }[];
  } catch (error) {
    console.error('Error parsing datasets', error);
    return [];
  }
};
