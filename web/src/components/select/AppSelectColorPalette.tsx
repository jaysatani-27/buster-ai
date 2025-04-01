'use client';
import React, { useMemo, useRef, useState } from 'react';
import { Button, Divider, Popover, Select } from 'antd';
import { CircleSpinnerLoaderContainer } from '../loaders/CircleSpinnerLoaderContainer';
import { timeout } from '@/utils';
import { useMemoizedFn, useMount } from 'ahooks';
import { createStyles } from 'antd-style';
import isEqual from 'lodash/isEqual';
import dynamic from 'next/dynamic';

const NewColorThemeModal = dynamic(() => import('./NewColorThemeModal'), {
  ssr: false,
  loading: () => (
    <div
      className="fixed bottom-0 left-0 right-0 top-0 z-10 h-full w-full"
      style={{
        background: `rgba(0, 0, 0, 0.125)`
      }}>
      <CircleSpinnerLoaderContainer />
    </div>
  )
});

const { Option } = Select;

export interface AppSelectColorPaletteProps {
  options: {
    id: string;
    colors: string[];
  }[];
  useCreateNewTheme?: boolean;
  value: string;
  onChange?: (id: string) => void;
  deleteTheme: (id: string) => Promise<void>;
  editTheme: (id: string, newTheme: string[]) => Promise<void>;
  createNewTheme: (newTheme: string[]) => Promise<void>;
  children?: React.ReactNode;
  appendedChildren?: React.ReactNode;
  open?: boolean;
  className?: string;
}

const useStyles = createStyles(({ token, css }) => {
  return {
    customOption: css`
      border-radius: ${token.borderRadius}px;
      height: ${token.controlHeight}px;
      &:hover {
        background: ${token.controlItemBgActive};
      }

      &.selected {
        background: ${token.controlItemBgActive};
      }
    `
  };
});

export const AppSelectColorPalette: React.FC<AppSelectColorPaletteProps> = ({
  options = [], //import { colorOptions } from '@/app/app/_controllers/ThreadController/ThreadControllerEditContent/SidebarChartApp';
  children,
  appendedChildren,
  onChange,
  useCreateNewTheme = true,
  className = '',
  createNewTheme,
  editTheme,
  deleteTheme,
  ...rest
}) => {
  const initOpenModal = useRef(false);
  const [openModal, setOpenModal] = React.useState(false);
  const [openSelect, setOpenSelect] = React.useState(false);
  const [selectedModalTheme, setSelectedModalTheme] = React.useState<string[] | null>(null);

  const onCreateTheme = useMemoizedFn(async (newTheme: string[]) => {
    return createNewTheme(newTheme);
  });

  const { defaultOptions, customOptions } = useMemo(() => {
    const defaultOptions = options.filter((option) => option.id.includes('default'));
    const customOptions = options.filter((option) => !option.id.includes('default'));
    return { defaultOptions, customOptions };
  }, [options]);

  const onEditTheme = useMemoizedFn(async (newTheme: string[], oldTheme: string[]) => {
    const id = options.find((p) => isEqual(p.colors, oldTheme))?.id;
    if (id !== undefined && id !== null) {
      await editTheme(id, newTheme);
    }
  });

  const onDeleteTheme = useMemoizedFn(async (id: string) => {
    await deleteTheme(id);
    if (id === rest.value) {
      onChange?.(options[0].id as string);
    }
  });

  return (
    <>
      <Select
        open={openSelect}
        onDropdownVisibleChange={(open) => {
          setOpenSelect(open);
        }}
        className={className + ' w-full'}
        defaultActiveFirstOption
        variant="outlined"
        {...rest}
        onChange={(ids: any) => {
          const lastId = ids; // ids[ids.length - 1];
          if (lastId !== null && lastId !== undefined && lastId) {
            onChange?.(lastId);
          }
        }}
        labelRender={(label) => {
          const assosciatedColor = options.find(
            (option) => (option.id as string) === (label.value as string)
          )?.colors;
          if (!assosciatedColor) return null;
          return <AppSelectColorPaletteRow colors={assosciatedColor!} />;
        }}
        dropdownRender={(menu) => (
          <>
            {menu}
            {useCreateNewTheme && (
              <>
                <Divider style={{ margin: '6px 0' }} />

                {customOptions.map((option) => (
                  <ColorOption
                    key={option.id}
                    {...{
                      option,
                      onChange,
                      onDeleteTheme,
                      setSelectedModalTheme,
                      setOpenModal,
                      initOpenModal,
                      setOpenSelect
                    }}
                  />
                ))}

                <Button
                  block
                  className="mt-1"
                  onClick={() => {
                    setOpenModal(true);
                    setOpenSelect(false);
                    initOpenModal.current = true;
                  }}>
                  Create New Theme
                </Button>
              </>
            )}
          </>
        )}>
        {!!children && children}
        {defaultOptions.map((option) => (
          <Option
            className="m-0 flex w-full flex-row !px-[0px] !py-[0px]"
            key={option.id}
            value={option.id}>
            <ColorOption
              option={option}
              {...{
                onChange,
                onDeleteTheme,
                setSelectedModalTheme,
                setOpenModal,
                initOpenModal,
                setOpenSelect
              }}
            />
          </Option>
        ))}
        {!!appendedChildren && appendedChildren}
      </Select>

      {initOpenModal.current && (
        <>
          <NewColorThemeModal
            open={openModal}
            onClose={() => setOpenModal(false)}
            modalTheme={selectedModalTheme}
            onCreateNewTheme={async (newTheme) => {
              await onCreateTheme?.(newTheme);
              setOpenModal(false);
            }}
            onEditTheme={async (newTheme) => {
              if (selectedModalTheme) {
                await onEditTheme(newTheme, selectedModalTheme);
                setOpenModal(false);
                setSelectedModalTheme(null);
              }
            }}
          />
        </>
      )}
    </>
  );
};

const AppSelectColorPaletteRow: React.FC<{ colors: string[] }> = ({ colors }) => {
  return (
    <div className="flex h-full w-full flex-row">
      {colors.map((color, id) => {
        return (
          <div
            key={color + id}
            className="h-full min-h-[18px] w-full first:rounded-l last:rounded-r"
            style={{ backgroundColor: color }}></div>
        );
      })}
    </div>
  );
};

const ColorOption: React.FC<{
  option: { id: string; colors: string[] };
  onChange?: (id: string) => void;
  onDeleteTheme?: (id: string) => void;
  setSelectedModalTheme: React.Dispatch<React.SetStateAction<string[] | null>>;
  setOpenModal: React.Dispatch<React.SetStateAction<boolean>>;
  initOpenModal: React.MutableRefObject<boolean>;
  isSelected?: boolean;
  setOpenSelect: React.Dispatch<React.SetStateAction<boolean>>;
}> = ({
  isSelected,
  onChange,
  onDeleteTheme,
  option,
  setSelectedModalTheme,
  setOpenModal,
  initOpenModal,
  setOpenSelect
}) => {
  const { cx, styles } = useStyles();
  const isDefaultOption = option.id.includes('default');

  return (
    <div
      key={option.id}
      className={cx(
        'cursor-pointer',
        styles.customOption,
        'm-0 flex w-full flex-row !px-[8px] !py-[3px]',
        isSelected ? 'selected' : ''
      )}>
      <CustomColorPopover
        onEditClick={() => {
          setSelectedModalTheme(option.colors);
          setOpenModal(true);
          setOpenSelect(false);
          initOpenModal.current = true;
        }}
        hideDelete={isDefaultOption}
        onDeleteClick={() => {
          onDeleteTheme?.(option.id as string);
        }}
        onClick={() => {
          onChange?.(option.id as string);
        }}>
        <AppSelectColorPaletteRow colors={option.colors} />
      </CustomColorPopover>
    </div>
  );
};

const BUSTER_SELECT_COLOR_PALETTE = 'buster-select-color-palette';
const CustomColorPopover: React.FC<{
  children: React.ReactNode;
  onDeleteClick: () => void;
  onEditClick: () => void;
  onClick: () => void;
  hideDelete?: boolean;
}> = ({ onClick, children, onDeleteClick, onEditClick, hideDelete = false }) => {
  const [hide, setHide] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useMount(() => {
    checkIfAncestorHasClass();
  });

  const checkIfAncestorHasClass = useMemoizedFn(async () => {
    const element = ref.current;
    if (element) {
      const t = !!element.closest(`.${BUSTER_SELECT_COLOR_PALETTE}`);
      setHide(t);
    }
  });

  const title = hideDelete ? 'Default Theme' : 'Edit Custom Theme';

  return (
    <div className="buster-popover h-full w-full" ref={ref}>
      <Popover
        trigger={['hover']}
        showArrow={true}
        title={<div className="font-normal">{title}</div>}
        zIndex={2000}
        destroyTooltipOnHide={true}
        onOpenChange={async (open) => {
          await timeout(1);
          if (open) checkIfAncestorHasClass();
        }}
        content={
          <div className="flex flex-col space-y-1">
            <Button
              size="small"
              block
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                onEditClick();
                setHide(true);
              }}>
              Edit
            </Button>
            {!hideDelete && (
              <Button
                size="small"
                block
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  onDeleteClick();
                  setHide(true);
                }}>
                Delete
              </Button>
            )}
          </div>
        }
        placement="right"
        align={{
          offset: [20, 0]
        }}
        overlayClassName={`w-[120px] ${hide ? 'opacity-0' : ''}`}>
        <div
          className="pointer-event-auto h-full"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onClick();
          }}>
          {children}
        </div>
      </Popover>
    </div>
  );
};
