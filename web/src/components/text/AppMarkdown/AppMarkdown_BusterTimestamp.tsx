import React, { useState } from 'react';
import { Text } from '../Text';
import { createStyles } from 'antd-style';
import { AppMaterialIcons } from '@/components/icons';
import { CircleSpinnerLoader } from '@/components/loaders';
import { useTimeout } from 'ahooks';
import { commonStreamingCheck, ExtraPropsExtra } from './AppMarkdownCommon';

export interface BusterTimestampProps extends ExtraPropsExtra {
  title: string;
  milliseconds?: string;
  id: string;
  showLoader: boolean;
  status: 'inProgress' | 'completed';
}

const MAX_TIME = 500;
const lineSize = 12;
const useStyles = (id: string) =>
  createStyles(({ token, css }) => ({
    container: css`
      &:has(+ .buster-timestamp) {
        .line {
          box-shadow: 0px 0px 0 0.5px ${token.colorTextTertiary};
          transform: translateX(5.5px);
          position: absolute;
          height: ${lineSize}px;
          bottom: -${lineSize + 1}px;
          opacity: 0.5;
        }
      }

      &[data-id='${id}'] {
        &:has(~ [data-id='${id}']) {
          display: none;
        }
      }

      &:nth-child(-n + 2):not(:has(+ .buster-timestamp)) {
        margin-top: 0px !important;
        margin-bottom: 10px !important;
      }

      &:nth-child(2):has(+ .buster-timestamp) {
        margin-top: 0px !important;
      }
    `,

    icon: css`
      color: ${token.colorIcon};
    `
  }));

export const BusterTimestamp: React.FC<BusterTimestampProps> = ({
  milliseconds = '0',
  title = '',
  id,
  status,
  numberOfLineMarkdown,
  node,
  showLoader,
  ...rest
}) => {
  // const [isTakingTooLong, setIsTakingTooLong] = useState(false);
  const { styles, cx } = useStyles(id)();
  const time = milliseconds ? `${(parseInt(milliseconds || '0') / 1000).toFixed(2)} seconds` : '';
  const isInProgress = status === 'inProgress';
  const iconSize = 11;

  const showStreamingLoader =
    node &&
    showLoader &&
    commonStreamingCheck(
      node?.position?.end.line,
      node?.position?.start.line,
      numberOfLineMarkdown - 2
    );

  // useTimeout(() => {
  //   if (showStreamingLoader) setIsTakingTooLong(true);
  // }, MAX_TIME);

  return (
    <div
      className={cx(styles.container, 'buster-timestamp mb-2.5 flex items-center space-x-1')}
      data-id={id}
      data-status={status}>
      <div className="relative flex">
        <div
          className="flex items-start justify-center"
          style={{ width: iconSize, height: iconSize }}>
          {isInProgress ? (
            <CircleSpinnerLoader size={8.5} />
          ) : (
            <AppMaterialIcons size={9} className={cx(styles.icon)} fill icon="check_circle" />
          )}
        </div>
        <div className="line"></div>
      </div>
      <div className="flex max-w-[85%] items-center space-x-1 overflow-hidden">
        <Text size="xs" ellipsis>
          {title}
        </Text>
        {time && (
          <Text size="xxs" className="!w-fit whitespace-nowrap" type="tertiary">
            {time}
          </Text>
        )}
      </div>
    </div>
  );
};

const tookTooLongMessage = (title: string) => {
  if (title.startsWith('SQL')) {
    return `Still ${title}`;
  }

  return `Still ${title.toLowerCase()}`;
};
