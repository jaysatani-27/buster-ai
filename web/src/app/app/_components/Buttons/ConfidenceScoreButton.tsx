import { AppMaterialIcons, AppPopover } from '@/components';
import { IBusterThreadMessage } from '@/context/Threads';
import { Text } from '@/components/text';
import { Button } from 'antd';
import { createStyles } from 'antd-style';
import React from 'react';

export const ConfidenceScoreButton: React.FC<{
  evaluation_score: NonNullable<IBusterThreadMessage>['evaluation_score'] | undefined;
  evaluation_summary: NonNullable<IBusterThreadMessage>['evaluation_summary'] | undefined;
  loading: boolean;
  disabled?: boolean;
}> = ({ evaluation_score, evaluation_summary, loading, disabled }) => {
  return (
    <AppPopover
      trigger={['click']}
      placement="bottomRight"
      content={
        disabled ? null : (
          <ConfidenceScorePopoverContent
            evaluation_score={evaluation_score}
            evaluation_summary={evaluation_summary}
            loading={loading}
          />
        )
      }>
      <div className={`flex`}>
        <ConfidenceScoreButtonComponent
          disabled={disabled}
          evaluation_score={evaluation_score}
          loading={loading}
        />
      </div>
    </AppPopover>
  );
};

type ConfidenceScore = 'High' | 'Moderate' | 'Low' | 'loading';

const ConfidenceScoreIconsRecord: Record<ConfidenceScore, string> = {
  loading: 'offline_bolt',
  Low: 'report',
  Moderate: 'warning',
  High: 'check_circle'
};

const ConfidenceScoreIcons: React.FC<{
  score: ConfidenceScore;
  size?: number;
  className?: string;
}> = ({ score, size, className }) => {
  const { styles, cx } = useStyles();
  const _score = score as ConfidenceScore;

  return (
    <div
      className={cx(
        styles[_score],
        'relative flex h-full w-full items-center transition',
        className
      )}>
      <AppMaterialIcons icon={ConfidenceScoreIconsRecord[_score] as 'report'} fill size={size} />
    </div>
  );
};

const ConfidenceScoreText: Record<ConfidenceScore, string> = {
  loading: 'Confidence Scores',
  Low: 'Low Confidence',
  Moderate: 'Medium Confidence',
  High: 'High Confidence'
};

const useStyles = createStyles(({ token, css }) => ({
  loading: css`
    color: ${token.colorTextDisabled};
  `,
  loadingIcon: css`
    color: ${token.colorTextDisabled};
    animation: pulse 2.75s ease-in-out infinite;
    @keyframes pulse {
      0% {
        opacity: 0.45;
      }
      50% {
        opacity: 1;
      }
      100% {
        opacity: 0.45;
      }
    }
  `,
  High: {
    color: token.colorSuccess
  },
  Moderate: {
    color: token.colorWarning
  },
  Low: {
    color: token.colorError
  }
}));

const ConfidenceScoreButtonComponent: React.FC<{
  evaluation_score: NonNullable<IBusterThreadMessage>['evaluation_score'] | undefined;
  loading: boolean;
  disabled?: boolean;
}> = ({ evaluation_score, loading, disabled }) => {
  const { styles, cx } = useStyles();
  const _score: ConfidenceScore = (
    evaluation_score && !loading ? evaluation_score : 'loading'
  ) as ConfidenceScore;

  return (
    <Button
      type="text"
      disabled={disabled}
      icon={
        <div className={cx(styles[_score], 'relative flex h-full w-full items-center transition')}>
          <ConfidenceScoreIcons score={_score} className={cx({ [styles.loadingIcon]: loading })} />
        </div>
      }
    />
  );
};

const fallbackText = `Buster will automatically run an evaluation on every SQL statement that is generated. You can reference this confidence score to get a better understanding of how accurate the SQL output is.`;

const ConfidenceScorePopoverContent: React.FC<{
  evaluation_score: NonNullable<IBusterThreadMessage>['evaluation_score'] | undefined;
  evaluation_summary: NonNullable<IBusterThreadMessage>['evaluation_summary'] | undefined;
  loading: boolean;
}> = ({ evaluation_score, evaluation_summary, loading }) => {
  const { styles, cx } = useStyles();
  const _score: ConfidenceScore = (evaluation_score ?? 'loading') as ConfidenceScore;
  const ConfidenceScoreIcon = ConfidenceScoreIcons({
    score: _score,
    size: 14
  });
  const text = ConfidenceScoreText[_score] ?? fallbackText;

  return (
    <div className="flex min-w-[300px] max-w-[300px] flex-col space-y-2 p-3">
      <div className={cx('flex justify-between', { hidden: loading })}>
        <div className={cx('flex items-center space-x-1', styles[_score])}>
          <div className={cx({ hidden: loading })}>{ConfidenceScoreIcon}</div>
          <Text type="inherit" size="base" className={cx(styles[_score])}>
            {text}
          </Text>
        </div>

        <AppPopover
          trigger={['click']}
          content={
            <div
              className="max-w-[250px]"
              style={{
                padding: 6
              }}>
              <Text type="secondary" size="sm">
                {fallbackText}
              </Text>
            </div>
          }>
          <AppMaterialIcons className="cursor-pointer" icon="help" size={14} />
        </AppPopover>
      </div>
      <Text type="secondary" size="sm">
        {evaluation_summary}
      </Text>
    </div>
  );
};
