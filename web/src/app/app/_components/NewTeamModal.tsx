import { Input, InputRef } from 'antd';
import React, { useEffect, useMemo, useRef, useState } from 'react';
import { useMemoizedFn } from 'ahooks';
import { usePermissionsContextSelector } from '@/context/Permissions';
import { AppModal } from '@/components/modal';

export const NewTeamModal: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  userId?: string;
}> = React.memo(({ isOpen, onClose, userId }) => {
  const inputRef = useRef<InputRef>(null);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [creatingTeam, setCreatingTeam] = useState(false);
  const createNewTeam = usePermissionsContextSelector((x) => x.createNewTeam);

  const disableSubmit = !title;

  const createNewTeamPreflight = useMemoizedFn(async () => {
    if (creatingTeam || disableSubmit) return;
    setCreatingTeam(true);
    const res = await createNewTeam({
      name: title,
      description
    });
    setTimeout(() => {
      onClose();
      setTitle('');
      setDescription('');
      setCreatingTeam(false);
    }, 250);
  });

  const onChangeTitle = useMemoizedFn((e: React.ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
  });

  const footer = useMemo(() => {
    return {
      primaryButton: {
        text: 'Create team',
        onClick: createNewTeamPreflight,
        loading: creatingTeam,
        disabled: disableSubmit
      }
    };
  }, [creatingTeam, disableSubmit]);

  useEffect(() => {
    if (isOpen)
      setTimeout(() => {
        inputRef.current?.focus();
      }, 150);
  }, [open]);

  return (
    <AppModal
      open={isOpen}
      onClose={onClose}
      header={{
        title: 'Create a team',
        description: `Once created, you'll be able to add team members to the team.`
      }}
      footer={footer}>
      <Input
        ref={inputRef}
        autoFocus
        placeholder="Team name..."
        value={title}
        onChange={onChangeTitle}
        onPressEnter={createNewTeamPreflight}
      />
    </AppModal>
  );
});
NewTeamModal.displayName = 'NewTeamModal';
