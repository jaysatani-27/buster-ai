import { createStyles } from 'antd-style';
import React, { useContext } from 'react';
import { Text } from '@/components/text/Text';
import { AppMaterialIcons } from '@/components/icons';
import {
  useBusterNewThreadsContextSelector,
  useBusterThreadsContextSelector
} from '@/context/Threads';

export interface BusterSuggestionProps {
  suggestions: string; //json {id: string, title: string, description?: string}[]
}

export const useStyles = createStyles(({ token, css }) => {
  return {
    container: css``,
    item: css`
      border-bottom: 0.5px solid ${token.colorBorder};
      cursor: pointer;

      &.buster-suggestion + .buster-suggestion {
        margin-top: 0 !important;
      }

      &:hover {
        background-color: #f5f5f5;
      }
    `
  };
});

export const BusterSuggestion: React.FC<BusterSuggestionProps> = ({ suggestions }) => {
  const onAskFollowUpQuestion = useBusterNewThreadsContextSelector((x) => x.onAskFollowUpQuestion);
  const selectedThreadId = useBusterThreadsContextSelector((x) => x.selectedThreadId);
  const { styles, cx } = useStyles();
  const suggestionsParsed = parseSuggestions(suggestions);

  const handleClick = async (id: string) => {
    await onAskFollowUpQuestion({
      threadId: selectedThreadId!,
      prompt: '',
      suggestion_id: id
    });
  };

  return (
    <div className={cx(styles.container, 'buster-suggestion')}>
      {suggestionsParsed.map((suggestion) => (
        <div
          key={suggestion.id}
          onClick={() => handleClick(suggestion.id)}
          data-id={suggestion.id}
          className={cx(styles.item, 'flex items-center justify-between space-x-2 py-2')}>
          <Text>{suggestion.title}</Text>
          <AppMaterialIcons icon="add" size={16} />
        </div>
      ))}
    </div>
  );
};

const parseSuggestions = (
  suggestions: string
): { id: string; title: string; description: string }[] => {
  try {
    return JSON.parse(suggestions) as { id: string; title: string; description: string }[];
  } catch (error) {
    return [];
  }
};
