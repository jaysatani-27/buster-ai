import React, { useCallback, useLayoutEffect, useState } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { AppMaterialIcons } from '../../../icons';
import { createStyles } from 'antd-style';
import { Button } from 'antd';
import darkTheme from 'react-syntax-highlighter/dist/cjs/styles/prism/vsc-dark-plus';
import { Text } from '@/components/text';
import { TextPulseLoader } from '../../..';

import { useAntToken } from '@/styles/useAntToken';
import lightTheme from './light';
import { useMemoizedFn } from 'ahooks';
import { useBusterNotifications } from '@/context/BusterNotifications';
import { useBusterStylesContext } from '@/context/BusterStyles/BusterStyles';

export const AppCodeBlock: React.FC<{
  language?: string;
  className?: string;
  children?: React.ReactNode;
  style?: React.CSSProperties;
  showLoader?: boolean;
  showCopyButton?: boolean;
}> = React.memo((props) => {
  const isDarkMode = useBusterStylesContext((state) => state.isDarkMode);
  const { children, className = '', language, showLoader, showCopyButton = true, ...rest } = props;
  const [style, setStyle] = useState<{
    [key: string]: React.CSSProperties;
  }>(lightTheme);
  const code = String(children).replace(/\n$/, '');

  useLayoutEffect(() => {
    const theme = isDarkMode ? darkTheme : lightTheme;
    setStyle(theme);
  }, [isDarkMode]);

  //this is a huge assumption, but if there is no language, it is probably an inline code block
  if (!language) {
    return <CodeInlineWrapper>{children}</CodeInlineWrapper>;
  }

  return (
    <CodeBlockWrapper
      code={code}
      isDarkMode={isDarkMode}
      language={language || ''}
      showCopyButton={showCopyButton}>
      <div className="w-full overflow-x-auto">
        <div className="code-wrapper">
          {language ? (
            <SyntaxHighlighter
              {...rest}
              className={`${className} !p-3 transition ${!style ? 'opacity-100' : '!m-0 !border-none !p-0 opacity-100'}`}
              children={code}
              language={language}
              style={style}
            />
          ) : (
            <code {...rest} className={className}>
              {children}
            </code>
          )}

          {showLoader && (
            <div className="-mt-2 pl-3">
              <TextPulseLoader />
            </div>
          )}
        </div>
      </div>
    </CodeBlockWrapper>
  );
});
AppCodeBlock.displayName = 'AppCodeBlock';

const useStyles = createStyles(({ token }) => ({
  container: {
    backgroundColor: token.colorBgBase,
    margin: `0px 0px`,
    border: `0.5px solid ${token.colorBorder}`,
    borderRadius: `${token.borderRadiusLG}px`,
    overflow: 'hidden'
  },
  containerHeader: {
    borderBottom: `0.5px solid ${token.colorBorder}`,
    padding: '4px',
    backgroundColor: token.controlItemBgActive
  },
  codeInlineWrapper: {
    backgroundColor: token.controlItemBgActive,
    borderRadius: token.borderRadiusSM,
    border: `0.5px solid ${token.colorBorder}`,
    fontSize: token.fontSize - 1
  }
}));

const CodeBlockWrapper: React.FC<{
  children: React.ReactNode;
  isDarkMode: boolean;
  code: string;
  language: string;
  showCopyButton: boolean;
}> = React.memo(({ children, code, showCopyButton, language }) => {
  const { cx, styles } = useStyles();
  const { openSuccessMessage } = useBusterNotifications();
  const token = useAntToken();

  const copyCode = useMemoizedFn(() => {
    navigator.clipboard.writeText(code);
    openSuccessMessage('Copied to clipboard');
  });

  return (
    <div className={cx(styles.container, 'max-h-fit')}>
      <div className={cx(styles.containerHeader, 'flex items-center justify-between')}>
        <Text className="pl-2">{language}</Text>
        {showCopyButton && (
          <Button
            style={{
              color: token.colorTextSecondary
            }}
            type="text"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              copyCode();
            }}
            icon={<AppMaterialIcons icon="content_copy" />}>
            Copy
          </Button>
        )}
      </div>

      {children}
    </div>
  );
});
CodeBlockWrapper.displayName = 'CodeBlockWrapper';

const CodeInlineWrapper: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const { cx, styles } = useStyles();
  return <code className={cx(styles.codeInlineWrapper, 'px-1')}>{children}</code>;
};
