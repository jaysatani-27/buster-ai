import { IBusterThread, IBusterThreadMessage } from '@/context/Threads/interfaces';
import React, { useLayoutEffect } from 'react';
import isEqual from 'lodash/isEqual';
import pick from 'lodash/pick';
import { useMemoizedFn } from 'ahooks';

export const useDisableSaveChanges = ({
  currentThreadMessage
}: {
  currentThreadMessage: IBusterThreadMessage | null;
  thread: IBusterThread | null;
}) => {
  const [initialThreadMessage, setInitialThreadMessage] =
    React.useState<IBusterThreadMessage | null>(currentThreadMessage);

  const disableSaveChangesButton = onCheckDisableSaveChanges({
    currentThreadMessage,
    initialThreadMessage
  });

  const showSaveButton = currentThreadMessage?.draft_session_id;

  const renderPreventNavigation =
    !!currentThreadMessage?.draft_session_id && !disableSaveChangesButton;

  const resetInitialThreadMessage = useMemoizedFn(() => {
    setInitialThreadMessage(currentThreadMessage);
  });

  useLayoutEffect(() => {
    if (currentThreadMessage?.id !== initialThreadMessage?.id) {
      resetInitialThreadMessage();
    }
  }, [currentThreadMessage?.id]);

  return {
    disableSaveChangesButton,
    showSaveButton,
    renderPreventNavigation,
    resetInitialThreadMessage
  };
};

const onCheckDisableSaveChanges = ({
  currentThreadMessage,
  initialThreadMessage
}: {
  currentThreadMessage: IBusterThreadMessage | null;
  initialThreadMessage: IBusterThreadMessage | null;
}): boolean => {
  if (!currentThreadMessage?.draft_session_id) return false;
  const keys: (keyof IBusterThreadMessage)[] = ['chart_config', 'title', 'code'];
  const pick1 = pick(currentThreadMessage, keys);
  const pick2 = pick(initialThreadMessage, keys);
  const check = isEqual(pick1, pick2);
  return check;
};
