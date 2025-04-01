import { Switch } from 'antd';
import React, { useLayoutEffect } from 'react';
import { AppModal, Text } from '@/components';
import { useAppLayoutContextSelector } from '@/context/BusterAppLayout';
import { BusterRoutes } from '@/routes';
import { useBusterNotifications } from '@/context/BusterNotifications';
import { useBusterNewThreadsContextSelector } from '@/context/Threads';

export const DuplicateChatModal: React.FC<{
  openId: string | null;
  threadId: string;
  onClose: () => void;
}> = React.memo(({ threadId, onClose, openId }) => {
  const [shareWithSamePeople, setShareWithSamePeople] = React.useState<boolean>(false);
  const { openSuccessMessage } = useBusterNotifications();
  const duplicateThread = useBusterNewThreadsContextSelector((x) => x.duplicateThread);
  const onChangePage = useAppLayoutContextSelector((s) => s.onChangePage);
  const [loading, setLoading] = React.useState<boolean>(false);
  const onMakeACopy = async () => {
    setLoading(true);
    const res = await duplicateThread({
      message_id: openId!,
      id: threadId,
      share_with_same_people: shareWithSamePeople
    });
    onChangePage({
      route: BusterRoutes.APP_THREAD_ID,
      threadId: res.id
    });
    setTimeout(() => {
      onClose();
      openSuccessMessage('Successfully duplicated the metric');
    }, 400);
  };

  useLayoutEffect(() => {
    if (openId) {
      setShareWithSamePeople(false);
    }
  }, [openId]);

  return (
    <AppModal
      open={!!openId}
      onClose={onClose}
      header={{
        title: 'Copy metric',
        description: `Would you like to make a copy of this metric?`
      }}
      footer={{
        primaryButton: {
          text: 'Make a copy',
          onClick: onMakeACopy,
          loading: loading,
          disabled: loading
        }
      }}>
      <div className="flex justify-between space-x-3">
        <Text type="secondary" size="md">
          Share it with the same people
        </Text>
        <Switch checked={shareWithSamePeople} onChange={setShareWithSamePeople} />
      </div>
    </AppModal>
  );
});
DuplicateChatModal.displayName = 'DuplicateChatModal';
