import React, { useState } from 'react';
import { AppMessageContainerMessage } from './AppMessagesContainer';
import { useAntToken } from '@/styles/useAntToken';
import { inputHasText } from '@/utils/text';
import { Button, Input, type InputRef } from 'antd';
import { useMount } from 'ahooks';

export const AppEditMessage: React.FC<{
  sentMessageText: string;
  id: AppMessageContainerMessage['id'];
  onSetEditMessageId: (id: string | null) => void;
  onEditMessage: (id: string, messageText: string) => Promise<void>;
}> = ({ onEditMessage, sentMessageText, id, onSetEditMessageId }) => {
  const token = useAntToken();
  const [messageText, setMessageText] = useState(sentMessageText);
  const canSave = inputHasText(messageText) && messageText !== sentMessageText;

  const inputRef = React.useRef<InputRef>(null);

  useMount(() => {
    setTimeout(() => {
      inputRef.current?.focus({
        cursor: 'all'
      });
    }, 50);
  });

  return (
    <div
      onClick={(e) => {
        e.stopPropagation();
        e.preventDefault();
      }}
      className="w-full space-y-3 overflow-hidden px-3 py-2"
      style={{
        border: `0.5px solid ${token.colorBorder}`,
        borderRadius: token.borderRadius,
        background: token.colorBgBase
      }}>
      <Input.TextArea
        ref={inputRef}
        className="overflow-hidden !px-0 !py-0"
        variant="borderless"
        style={{
          transition: 'none'
        }}
        value={messageText}
        autoSize={{ minRows: 1, maxRows: 10 }}
        placeholder={'Edit this message...'}
        onChange={(e) => setMessageText(e.target.value)}
      />

      <div className="flex w-full justify-end space-x-1">
        <Button
          type="text"
          onClick={() => {
            onSetEditMessageId(null);
          }}>
          Cancel
        </Button>
        <Button
          disabled={!canSave}
          type="default"
          onClick={() => {
            if (!canSave) return;
            onEditMessage(id, messageText);
          }}>
          Save
        </Button>
      </div>
    </div>
  );
};
