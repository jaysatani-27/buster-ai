import React, { useState } from 'react';
import { AppMessageContainerMessage } from './AppMessagesContainer';
import { createStyles } from 'antd-style';
import { BusterAvatar, BusterUserAvatar } from '@/components/image';
import { AppEditMessage } from './AppEditMessage';
import { Paragraph } from '@/components/text';
import { AppMarkdown } from '@/components/text/AppMarkdown';
import { AppMessageTooltip } from './AppMessageTooltip';
import { AppMessageThoughts } from './AppMessageThoughts';
import { IBusterThreadMessage } from '@/context/Threads/interfaces';
import { useMemoizedFn } from 'ahooks';

const useStyles = createStyles(({ token, css }) => ({
  messageContainer: css`
    border-top: 0.5px solid transparent;
    border-bottom: 0.5px solid transparent;
    transition:
      background-color 0.2s,
      border 0.2s;
    &:hover {
      background-color: ${token.controlItemBgHover};
    }
    &.isSelected {
      border-color: ${token.colorBorder};
      background-color: ${token.controlItemBgActive};
    }
  `
}));

export interface AppHistoryMessageProps {
  message: AppMessageContainerMessage;
  isReadOnly: boolean;
  isStreaming: boolean;
  openMessageId: string | null;
  editMessageId: string | null;
  thoughts: IBusterThreadMessage['thoughts'];
  onSetOpenMessageId: (id: string) => void;
  onEditMessage: (id: string, messageText: string) => Promise<void>;
  onSetEditMessageId: (id: string | null) => void;
}

const MESSAGE_CONTAINER_STYLE = {
  scrollMarginTop: '10px'
};

export const AppHistoryMessage: React.FC<AppHistoryMessageProps> = ({
  message,
  isReadOnly,
  onSetEditMessageId,
  onEditMessage,
  editMessageId,
  openMessageId,
  onSetOpenMessageId,
  isStreaming,
  thoughts
}) => {
  const { styles, cx } = useStyles();
  const [showTooltip, setShowTooltip] = useState(false);

  const { id, sentMessage, responseMessage, tooltipItems } = message;
  const isEditingMessage = editMessageId === id;

  const onMessageClick = useMemoizedFn(() => onSetOpenMessageId(id));
  const onMessageMouseEnter = useMemoizedFn(() => setShowTooltip(true));
  const onMessageMouseLeave = useMemoizedFn(() => setShowTooltip(false));

  return (
    <div
      id={'message_' + id}
      style={MESSAGE_CONTAINER_STYLE}
      onClick={onMessageClick}
      onMouseEnter={onMessageMouseEnter}
      onMouseLeave={onMessageMouseLeave}
      className={cx(
        'sidebar-history-message',
        'group relative flex w-full cursor-pointer flex-col space-y-4 px-3 py-2',
        styles.messageContainer,
        openMessageId === id && 'isSelected'
      )}>
      <UserMessage
        sentMessageName={sentMessage.name}
        sentMessageText={sentMessage.text}
        id={id}
        editMessageId={editMessageId}
        onSetEditMessageId={onSetEditMessageId}
        onEditMessage={onEditMessage}
      />

      <AssistantMessage
        id={id}
        responseMessageText={responseMessage.text}
        isStreaming={isStreaming}
        thoughts={thoughts}
      />

      {!isReadOnly && !isStreaming && (
        <AppMessageTooltip
          showTooltip={showTooltip && !isEditingMessage}
          tooltipItems={tooltipItems}
        />
      )}
    </div>
  );
};

const AssistantMessage: React.FC<{
  isStreaming: boolean;
  thoughts: AppHistoryMessageProps['thoughts'];
  responseMessageText: AppMessageContainerMessage['responseMessage']['text'];

  id: string;
}> = ({ id, responseMessageText, isStreaming, thoughts }) => {
  const { cx } = useStyles();

  return (
    <div className={cx('flex space-x-2 overflow-x-hidden')}>
      <BusterAvatar size={24} />

      <div className="relative flex w-full flex-col space-y-1.5 overflow-x-hidden">
        <AppMessageThoughts {...thoughts} key={id} />
        <AppMarkdown showLoader={isStreaming} markdown={responseMessageText} />
      </div>
    </div>
  );
};

const UserMessage: React.FC<{
  editMessageId: string | null;
  onSetEditMessageId: (id: string | null) => void;
  onEditMessage: (id: string, messageText: string) => Promise<void>;
  sentMessageName: string;
  sentMessageText: string;
  id: string;
}> = React.memo(
  ({ id, sentMessageName, sentMessageText, editMessageId, onSetEditMessageId, onEditMessage }) => {
    return (
      <div className={'flex space-x-2'}>
        <BusterUserAvatar size={24} name={sentMessageName} />

        {editMessageId === id ? (
          <AppEditMessage
            sentMessageText={sentMessageText}
            id={id}
            onSetEditMessageId={onSetEditMessageId}
            onEditMessage={onEditMessage}
          />
        ) : (
          <Paragraph className="!mb-0">{sentMessageText}</Paragraph>
        )}
      </div>
    );
  }
);
UserMessage.displayName = 'UserMessage';
