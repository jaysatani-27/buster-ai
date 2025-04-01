import React, { useEffect } from 'react';
import { Input, InputRef } from 'antd';
import { AppModal } from '@/components';
import { useCollectionsContextSelector } from '@/context/Collections';
import { useAppLayoutContextSelector } from '@/context/BusterAppLayout';
import { BusterRoutes } from '@/routes';
import { inputHasText } from '@/utils';
import { useMemoizedFn } from 'ahooks';

export const NewCollectionModal: React.FC<{
  open: boolean;
  onClose: () => void;
  useChangePage?: boolean;
  onCollectionCreated?: (collectionId: string) => Promise<void>;
}> = React.memo(({ onCollectionCreated, onClose, open, useChangePage = true }) => {
  const [title, setTitle] = React.useState('');
  const onChangePage = useAppLayoutContextSelector((s) => s.onChangePage);
  const createNewCollection = useCollectionsContextSelector((x) => x.createNewCollection);
  const creatingCollection = useCollectionsContextSelector((x) => x.creatingCollection);
  const inputRef = React.useRef<InputRef>(null);
  const disableSubmit = !inputHasText(title);

  const onCreateNewCollection = useMemoizedFn(async () => {
    if (creatingCollection || disableSubmit) return;
    const res = await createNewCollection({ name: title, onCollectionCreated });
    if (useChangePage) {
      onChangePage({
        route: BusterRoutes.APP_COLLECTIONS_ID,
        collectionId: (res as any).id
      });
    }
    setTimeout(() => {
      onClose();
    }, 200);
  });

  useEffect(() => {
    if (open) {
      setTimeout(() => {
        inputRef.current?.focus();
      }, 200);
    }
  }, [open]);

  return (
    <AppModal
      open={open}
      onClose={onClose}
      header={{
        title: 'New collection',
        description: 'Once created, you will be able to add dashboards and metric to the collection'
      }}
      footer={{
        primaryButton: {
          text: 'Create a collection',
          onClick: onCreateNewCollection,
          loading: creatingCollection,
          disabled: disableSubmit
        }
      }}>
      <Input
        ref={inputRef}
        placeholder="Collection title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onPressEnter={onCreateNewCollection}
      />
    </AppModal>
  );
});
NewCollectionModal.displayName = 'NewCollectionModal';
