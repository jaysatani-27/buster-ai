import React from 'react';
import { AppMessagesFollowup } from './AppMessagesFollowup';
import { AppMessagesHistory } from './AppMessagesHistory';
import { IBusterThreadMessage } from '@/context/Threads';

export interface AppMessageContainerMessage {
  id: string;
  sentMessage: {
    text: string;
    name: string;
    avatar?: string;
  };
  responseMessage: {
    text: string;
    avatar?: string;
    name?: string;
  };
  tooltipItems?: {
    tooltipText: string;
    icon: React.ReactNode;
    onClick: () => void;
    selected?: boolean;
    shortcuts?: string[];
  }[];
  thoughts: IBusterThreadMessage['thoughts'];
}

export const AppMessagesContainer: React.FC<{
  isReadOnly: boolean;
  className?: string;
  showSkeletonLoader: boolean;
  onSendMessage: (message: string) => Promise<void>;
  messages: AppMessageContainerMessage[];
  openMessageId: string | null;
  editMessageId: string | null;
  onSetEditMessageId: (id: string | null) => void;
  onSetOpenMessageId: (id: string) => void;
  onEditMessage: (id: string, messageText: string) => Promise<void>;
  isStreaming: boolean;
  inputPlaceholder: string;
}> = ({
  openMessageId,
  editMessageId,
  isReadOnly,
  className,
  showSkeletonLoader,
  onSendMessage,
  messages,
  isStreaming,
  onSetEditMessageId,
  onSetOpenMessageId,
  onEditMessage,
  inputPlaceholder
}) => {
  return (
    <div className={`relative flex h-full w-full flex-col justify-between ${className}`}>
      <AppMessagesHistory
        isReadOnly={isReadOnly}
        messages={messages}
        openMessageId={openMessageId}
        editMessageId={editMessageId}
        onSetEditMessageId={onSetEditMessageId}
        onSetOpenMessageId={onSetOpenMessageId}
        onEditMessage={onEditMessage}
        isStreaming={isStreaming}
      />

      {!isReadOnly && (
        <AppMessagesFollowup
          showSkeletonLoader={showSkeletonLoader}
          placeholder={inputPlaceholder}
          onSend={onSendMessage}
          disabled={isStreaming}
        />
      )}
    </div>
  );
};
