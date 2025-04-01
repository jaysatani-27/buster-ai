import { AppMaterialIcons } from '@/components/icons';
import { AppCodeEditor } from '@/components/inputs/AppCodeEditor';
import { useBusterNotifications } from '@/context/BusterNotifications';
import { useMemoizedFn } from 'ahooks';
import { Button, Divider } from 'antd';
import { createStyles } from 'antd-style';
import React, { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';

export const SQLContainer: React.FC<{
  className?: string;
  datasetSQL: string | undefined;
  setDatasetSQL: (sql: string) => void;
  onRunQuery: () => Promise<void>;
  error?: string;
}> = ({ className = '', datasetSQL, setDatasetSQL, onRunQuery, error }) => {
  const { styles, cx } = useStyles();
  const [isRunning, setIsRunning] = useState(false);
  const [isError, setIsError] = useState(false);
  const { openInfoMessage } = useBusterNotifications();

  const onCopySQL = useMemoizedFn(() => {
    navigator.clipboard.writeText(datasetSQL || '');
    openInfoMessage('SQL copied to clipboard');
  });

  const onRunQueryPreflight = useMemoizedFn(async () => {
    setIsRunning(true);
    await onRunQuery();
    setIsRunning(false);
  });

  useEffect(() => {
    setIsError(!!error);
  }, [error]);

  return (
    <div className={cx(styles.container, 'flex h-full w-full flex-col overflow-hidden', className)}>
      <AppCodeEditor
        className="overflow-hidden"
        value={datasetSQL}
        onChange={setDatasetSQL}
        onMetaEnter={onRunQueryPreflight}
      />
      <Divider className="!my-0" />
      <div className="relative flex items-center justify-between px-4 py-2.5">
        <Button type="default" onClick={onCopySQL}>
          Copy SQL
        </Button>

        <Button
          type="default"
          loading={isRunning}
          disabled={!datasetSQL}
          className="flex items-center space-x-0"
          onClick={onRunQueryPreflight}>
          <span>Run</span>
          <AppMaterialIcons icon="keyboard_command_key" />
          <AppMaterialIcons icon="keyboard_return" />
        </Button>

        {error && (
          <ErrorContainer error={error} onClose={() => setIsError(false)} isError={isError} />
        )}
      </div>
    </div>
  );
};

const ErrorContainer: React.FC<{
  error: string;
  onClose: () => void;
  isError: boolean;
}> = ({ error, onClose, isError }) => {
  const { styles, cx } = useStyles();

  return (
    <AnimatePresence mode="wait">
      {isError && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 0 }}
          className={cx(styles.errorContainer, 'absolute bottom-full left-0 right-0 mx-4 mb-2')}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <AppMaterialIcons icon="error" />
              <span>{error}</span>
            </div>
            <button
              onClick={() => onClose()}
              className={cx(
                styles.closeButton,
                'rounded p-0.5 transition-colors hover:bg-black/5'
              )}>
              <AppMaterialIcons icon="close" className="text-sm" />
            </button>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    background: ${token.colorBgBase};
    border-radius: ${token.borderRadius}px;
    border: 0.5px solid ${token.colorBorder};
  `,
  errorContainer: css`
    background: ${token.colorErrorBg};
    color: ${token.colorError};
    border: 0.5px solid ${token.colorError};
    padding: 8px 12px;
    border-radius: ${token.borderRadius}px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  `,
  closeButton: css`
    color: ${token.colorError};
    cursor: pointer;
    border: none;
    background: none;
    display: flex;
    align-items: center;
    justify-content: center;

    &:hover {
      opacity: 0.8;
    }
  `
}));
