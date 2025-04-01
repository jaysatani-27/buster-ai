import { AppCodeEditor } from '@/components/inputs/AppCodeEditor';
import { createStyles } from 'antd-style';
import React from 'react';

export const MetadataContainer: React.FC<{
  ymlFile: string;
  setYmlFile: (ymlFile: string) => void;
}> = React.memo(({ ymlFile, setYmlFile }) => {
  const { styles, cx } = useStyles();

  return (
    <div className={cx(styles.container, 'flex h-full w-full flex-col overflow-hidden')}>
      <AppCodeEditor
        language="yaml"
        className="overflow-hidden"
        value={ymlFile}
        onChange={setYmlFile}
      />
    </div>
  );
});

MetadataContainer.displayName = 'MetadataContainer';

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    background: ${token.colorBgBase};
    border-radius: ${token.borderRadius}px;
    border: 0.5px solid ${token.colorBorder};
  `
}));
