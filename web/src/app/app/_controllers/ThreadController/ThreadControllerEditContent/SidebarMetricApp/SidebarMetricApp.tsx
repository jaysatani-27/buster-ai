import { AppMaterialIcons } from '@/components';
import {
  useBusterNewThreadsContextSelector,
  useBusterThreadIndividual,
  useBusterThreadMessage,
  useBusterThreads,
  useBusterThreadsContextSelector
} from '@/context/Threads';
import React, { useMemo, useRef, useState } from 'react';
import { inputHasText } from '@/utils';
import { useMemoizedFn, useWhyDidYouUpdate } from 'ahooks';
import { IBusterThread, IBusterThreadMessage } from '@/context/Threads/interfaces';
import { DuplicateChatModal } from './DuplicateChatModal';
import {
  AppMessageContainerMessage,
  AppMessagesContainer
} from '@/components/messages/AppMessagesContainer';
import { useBusterNotifications } from '@/context/BusterNotifications';

export const SidebarMetricApp: React.FC<{
  showSkeletonLoader?: boolean;
  isReadOnly: boolean;
}> = React.memo(({ showSkeletonLoader, isReadOnly }) => {
  const { openInfoMessage } = useBusterNotifications();
  const getThreadCurrentMessageStale = useBusterThreadsContextSelector(
    (x) => x.getThreadMessageNotLiveDataMethodOnly
  );
  const onSetCurrentMessageId = useBusterThreadsContextSelector((x) => x.onSetCurrentMessageId);
  const selectedThreadId = useBusterThreadsContextSelector((x) => x.selectedThreadId);
  const onUpdateThreadMessage = useBusterThreadsContextSelector((x) => x.onUpdateThreadMessage);
  const onAskFollowUpQuestion = useBusterNewThreadsContextSelector((x) => x.onAskFollowUpQuestion);
  const onEditMessage = useBusterNewThreadsContextSelector((x) => x.onEditMessage);
  const { thread } = useBusterThreadIndividual({ threadId: selectedThreadId });
  const selectedMessageId = useBusterThreadsContextSelector(
    (x) => x.currentMessageIdByThread[selectedThreadId]
  );
  const selectedMessage = useBusterThreadMessage({
    threadId: selectedThreadId,
    messageId: selectedMessageId || undefined
  });

  const [editMessageId, setEditMessageId] = useState<string | null>(null);
  const [isAskingFollowUp, setIsAskingFollowUp] = useState(false);
  const [openDuplicateChatId, setOpenDuplicateChatId] = useState<string | null>(null);
  const tooltipItemsState = useRef<{
    [key: string]: AppMessageContainerMessage['tooltipItems'];
  }>({});

  const isStreaming = thread?.isNewThread || thread?.isFollowupMessage;

  const onSetEditMessageId = useMemoizedFn((id: string | null) => {
    setEditMessageId(id);
  });

  const onDuplicateChat = useMemoizedFn((id: string) => {
    setOpenDuplicateChatId(id);
  });

  const onThumbsDownMessage = useMemoizedFn((id: string) => {
    const currentMessage = getThreadCurrentMessageStale({
      threadId: selectedThreadId,
      messageId: id
    });
    const feedback = currentMessage?.feedback === 'negative' ? null : 'negative';
    onUpdateThreadMessage({
      threadId: selectedThreadId,
      messageId: id,
      message: {
        feedback
      }
    });
    openInfoMessage('Feedback submitted');
  });

  const onAskFollowUpQuestionPreflight = useMemoizedFn(async (inputValue: string) => {
    if (!inputHasText(inputValue) || isAskingFollowUp) return;
    setIsAskingFollowUp(true);
    try {
      await onAskFollowUpQuestion({
        threadId: selectedThreadId,
        prompt: inputValue
      });
    } catch (error) {
      //
    }
    setIsAskingFollowUp(false);
  });

  const onEditMessagePreflight = useMemoizedFn(async (id: string, messageText: string) => {
    const selectedMessage = getThreadCurrentMessageStale({ threadId: thread.id, messageId: id });
    if (!selectedMessage) return;
    setIsAskingFollowUp(true);
    onSetEditMessageId(null);
    await onEditMessage({ threadId: selectedThreadId, messageId: id, prompt: messageText });
    setIsAskingFollowUp(false);
  });

  const messages: AppMessageContainerMessage[] = useMemo(() => {
    return (
      thread?.messages.map((messageId) => {
        const message = getThreadCurrentMessageStale({ threadId: selectedThreadId, messageId });
        const tooltipItemsKey = `${message.id}-${selectedMessageId}`;

        const tooltipItems =
          tooltipItemsState.current[tooltipItemsKey] ||
          createTooltipItems({
            message,
            thread,
            selectedMessageId,
            onSetEditMessageId,
            onSetCurrentMessageId,
            onDuplicateChat,
            onThumbsDownMessage
          });

        tooltipItemsState.current[tooltipItemsKey] = tooltipItems;

        return {
          id: message.id,
          thoughts: message.thoughts,
          sentMessage: {
            text: message.message,
            name: message.sent_by_name
          },
          responseMessage: {
            text: message.response
          },
          tooltipItems
        };
      }) || []
    );
  }, [selectedMessage?.message, thread?.messages, selectedMessageId]);

  const onSetOpenMessageId = useMemoizedFn((messageId: string | null) => {
    onSetCurrentMessageId({ threadId: selectedThreadId, messageId });
  });

  return (
    <>
      <AppMessagesContainer
        isReadOnly={isReadOnly}
        showSkeletonLoader={!!showSkeletonLoader}
        onSendMessage={onAskFollowUpQuestionPreflight}
        openMessageId={selectedMessageId}
        editMessageId={editMessageId}
        onSetEditMessageId={onSetEditMessageId}
        isStreaming={isStreaming}
        onEditMessage={onEditMessagePreflight}
        onSetOpenMessageId={onSetOpenMessageId}
        messages={messages}
        inputPlaceholder="Ask a follow up question..."
      />

      {!isReadOnly && (
        <DuplicateChatModal
          openId={openDuplicateChatId}
          threadId={selectedThreadId}
          onClose={() => setOpenDuplicateChatId(null)}
        />
      )}
    </>
  );
});
SidebarMetricApp.displayName = 'SidebarMetricApp';

const createTooltipItems = ({
  message,
  onSetEditMessageId,
  thread,
  onSetCurrentMessageId,
  onDuplicateChat,
  onThumbsDownMessage,
  selectedMessageId
}: {
  message: IBusterThreadMessage;
  thread: IBusterThread;
  onSetEditMessageId: (id: string | null) => void;
  onSetCurrentMessageId: ReturnType<typeof useBusterThreads>['onSetCurrentMessageId'];
  onDuplicateChat: (id: string) => void;
  onThumbsDownMessage: (id: string) => void;
  selectedMessageId: string | null;
}): AppMessageContainerMessage['tooltipItems'] => {
  return [
    {
      tooltipText: 'View this message',
      icon: <AppMaterialIcons icon="keep" />,
      selected: selectedMessageId === message.id,
      onClick: () => {
        onSetCurrentMessageId({ threadId: thread.id, messageId: message.id });
      }
    },
    {
      tooltipText: 'Edit',
      icon: <AppMaterialIcons icon="edit" />,
      onClick: () => {
        onSetEditMessageId(message.id);
      }
    },
    {
      tooltipText: 'Duplicate this chat',
      icon: <AppMaterialIcons icon="post_add" />,
      onClick: () => {
        onDuplicateChat(message.id);
      }
    },
    {
      tooltipText: 'Data is incorrect',
      icon: <AppMaterialIcons icon="thumb_down" />,
      onClick: () => {
        onThumbsDownMessage(message.id);
      }
    }
  ];
};
