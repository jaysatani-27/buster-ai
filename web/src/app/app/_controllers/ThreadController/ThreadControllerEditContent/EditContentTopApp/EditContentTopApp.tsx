import React, { useMemo } from 'react';
import { SidebarApplications } from '../config';
import { AppMaterialIcons, AppSegmented, AppTooltip, PreventNavigation } from '@/components';
import { ConfigProvider, Divider, ThemeConfig } from 'antd';
import { IBusterThread } from '@/context/Threads/interfaces';
import { useMemoizedFn } from 'ahooks';
import { SegmentedValue } from 'antd/es/segmented';
import { UndoRedoContainer } from './UndoRedoContainer';
import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';
import { useSQLContextSelector } from '@/context/SQL';
import { useBusterNotifications } from '@/context/BusterNotifications';
import { useBusterCurrentThreadMessage } from '@/context/Threads';
const token = busterAppStyleConfig.token!;

const options = [
  {
    name: SidebarApplications.Copilot,
    icon: <AppMaterialIcons icon="table" />,
    hotkey: 'C'
  },
  {
    name: SidebarApplications.Styling,
    icon: <AppMaterialIcons icon="monitoring" />,
    hotkey: 'S'
  },
  {
    name: SidebarApplications.SQL,
    icon: <AppMaterialIcons icon="data_object" />,
    hotkey: 'Q'
  }
];

const segmentedTheme: ThemeConfig = {
  components: {
    Segmented: {
      itemColor: token.colorTextDescription,
      trackBg: 'transparent',
      itemSelectedBg: token.controlItemBgActive,
      colorBorder: token.colorBorder,
      boxShadowTertiary: 'none'
    }
  }
};

export const EditContentTopApp: React.FC<{
  isReadOnly: boolean;
  selectedApp: SidebarApplications;
  setSelectedApp: (app: SidebarApplications) => void;
  disabled: boolean;
  threadId: IBusterThread['id'];
}> = React.memo(({ disabled, threadId, selectedApp, setSelectedApp, isReadOnly }) => {
  const warnBeforeNavigating = useSQLContextSelector((x) => x.warnBeforeNavigating);
  const resetRunSQLData = useSQLContextSelector((x) => x.resetRunSQLData);
  const currentThreadMessageId = useBusterCurrentThreadMessage({ threadId });
  const { openConfirmModal } = useBusterNotifications();

  const segmentedOptions = useMemo(() => {
    return options
      .map((option) => {
        return {
          value: option.name,
          label: option.name
        };
      })
      .filter((v) => {
        if (isReadOnly) return v.value === SidebarApplications.Copilot;
        return true;
      });
  }, [isReadOnly]);

  const selectedAppOption = useMemo(() => {
    return segmentedOptions.find((option) => option.value === selectedApp) || segmentedOptions[0];
  }, [segmentedOptions, selectedApp]);

  const resetSQLPreflight = useMemoizedFn(async () => {
    if (warnBeforeNavigating && currentThreadMessageId) {
      resetRunSQLData({
        threadId: threadId,
        messageId: currentThreadMessageId
      });
    }
  });

  const onChangeSegmented = useMemoizedFn((v: SegmentedValue) => {
    const method = () => {
      setSelectedApp(v as SidebarApplications);
      resetSQLPreflight();
    };
    if (warnBeforeNavigating) {
      openConfirmModal({
        title: 'Navigate pages',
        content: 'You will lose your current changes.',
        onOk: method,
        useReject: false
      });
    } else {
      method();
    }
  });

  return (
    <React.Fragment>
      <div className="flex flex-col">
        <div className="flex w-full items-center justify-between space-x-2 px-3 py-1">
          <ConfigProvider theme={segmentedTheme}>
            <AppSegmented
              options={segmentedOptions}
              value={selectedAppOption?.value}
              onChange={onChangeSegmented}
              disabled={disabled}
            />
          </ConfigProvider>

          {!isReadOnly && <UndoRedoContainer threadId={threadId} />}
        </div>

        <Divider className="!mb-0 !mt-0" />
      </div>
    </React.Fragment>
  );
});
EditContentTopApp.displayName = 'EditContentTopApp';
