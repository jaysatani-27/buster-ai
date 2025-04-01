import * as antColors from '@ant-design/colors';
import React, { useEffect, useState } from 'react';
import { Button, ColorPicker, Modal } from 'antd';
import { useMemoizedFn } from 'ahooks';
import pick from 'lodash/pick';
import shuffle from 'lodash/shuffle';
import { AppTooltip } from '../tooltip';
import { AppMaterialIcons } from '../icons';
import { Text } from '@/components';

export const NewColorThemeModal: React.FC<{
  open: boolean;
  onClose: () => void;
  modalTheme: string[] | null;
  onEditTheme: (theme: string[]) => Promise<void>;
  onCreateNewTheme: (theme: string[]) => Promise<void>;
}> = ({ open, onCreateNewTheme, modalTheme, onEditTheme, onClose }) => {
  const onRandomize = useMemoizedFn((): string[] => {
    const selectedPallets = pick(antColors, [
      'blue',
      'red',
      'green',
      'purple',
      'gold',
      'orange',
      'volcano',
      'magenta'
    ]);

    const randomColors = Object.values(selectedPallets).reduce((acc, curr) => {
      const middleColors = curr.slice(2, 8);
      const randomColor = middleColors[Math.floor(Math.random() * middleColors.length)]!;
      return [...acc, randomColor];
    }, [] as string[]);

    const allColors = Object.values(selectedPallets).flatMap((p) => p.slice(1, 8));
    const numberOfColors = 10;
    const numberOfColorsShort = numberOfColors - randomColors.length;
    for (let i = 0; i < numberOfColorsShort; i++) {
      const randomColor = allColors[Math.floor(Math.random() * allColors.length)]!;
      randomColors.push(randomColor);
    }

    const shuffleColors = shuffle(randomColors);

    return shuffleColors;
  });

  const [theme, setTheme] = useState<string[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const onSubmitPreflight = async () => {
    setSubmitting(true);

    if (modalTheme) {
      await onEditTheme(theme);
    } else {
      await onCreateNewTheme(theme);
    }
    setSubmitting(false);
  };

  useEffect(() => {
    if (open && modalTheme && modalTheme.length > 0) {
      setTheme(modalTheme);
    } else if (open) {
      setTheme(onRandomize());
    }
  }, [open]);

  return (
    <Modal
      destroyOnClose
      open={open}
      footer={[
        <div key="footer" className="flex w-full justify-between">
          <AppTooltip title={'Generate Palette'} trigger={['hover']}>
            <Button
              onClick={() => {
                setTheme(onRandomize());
              }}
              icon={<AppMaterialIcons icon="science" />}></Button>
          </AppTooltip>

          <Button
            loading={submitting}
            onClick={async () => {
              onSubmitPreflight();
            }}>
            {modalTheme ? 'Save theme' : 'Create theme'}
          </Button>
        </div>
      ]}
      onCancel={onClose}
      width={730}
      title="Create theme">
      <Text className="">
        It is recommended to choose a color pallete with a wide range of colors as it will be used
        for charts.
      </Text>

      <div className="mt-5 flex">
        {theme?.map((color, index) => {
          return (
            <ColorPicker
              key={index}
              value={color}
              disabledAlpha
              destroyTooltipOnHide
              onChange={(v) => {
                const newTheme = [...theme];
                newTheme[index] = v.toHexString();
                setTheme(newTheme);
              }}>
              <div
                onClick={() => {}}
                className="h-full min-h-[118px] w-full cursor-pointer transition first:rounded-l last:rounded-r hover:z-10 hover:scale-105 hover:shadow-lg"
                style={{ backgroundColor: color }}></div>
            </ColorPicker>
          );
        })}
      </div>
    </Modal>
  );
};

export default NewColorThemeModal;
