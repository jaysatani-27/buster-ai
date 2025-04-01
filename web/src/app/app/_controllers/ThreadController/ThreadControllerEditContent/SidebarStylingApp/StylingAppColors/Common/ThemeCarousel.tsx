import React, { useEffect, useMemo, useRef, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { AppMaterialIcons } from '@/components/icons';
import { IColorTheme } from './interfaces';
import { useMemoizedFn, useSize } from 'ahooks';
import { createStyles } from 'antd-style';
import { ThemeColorDots } from './ThemeColorDots';

type IThemeCarouselItem = IColorTheme & {
  selected?: boolean;
};

interface IThemeCarouselProps {
  themes: IThemeCarouselItem[];
  showTitle?: boolean;
  onChange: (theme: IThemeCarouselItem) => void;
}

const MIN_WIDTH_FOR_TITLE = 300;
let lastSeenWidth = MIN_WIDTH_FOR_TITLE;

export const ThemeCarousel: React.FC<IThemeCarouselProps> = React.memo(
  ({ themes, showTitle: showTitleProp = true, onChange }) => {
    const { styles, cx } = useStyles();
    const containerRef = useRef<HTMLDivElement>(null);
    const width = useSize(containerRef)?.width || lastSeenWidth;

    const showTitle = useMemo(() => {
      lastSeenWidth = width;
      return width > MIN_WIDTH_FOR_TITLE && showTitleProp;
    }, [width, showTitleProp]);

    const indexesPerItem = useMemo(() => {
      return showTitle ? 3 : 5;
    }, [showTitle]);

    const initialIndex = useMemo(() => {
      const selectedIndex = themes.findIndex((theme) => theme.selected);
      if (selectedIndex === -1) return 0;
      return Math.floor(selectedIndex / indexesPerItem) * indexesPerItem;
    }, [themes.length, indexesPerItem]);

    const [currentIndex, setCurrentIndex] = useState(initialIndex);

    const handleNext = useMemoizedFn(() => {
      setCurrentIndex((prev) => {
        const nextIndex = prev + indexesPerItem;
        if (nextIndex >= themes.length) return 0;
        return nextIndex;
      });
    });

    const handlePrev = useMemoizedFn(() => {
      setCurrentIndex((prev) => {
        const prevIndex = prev - indexesPerItem;
        if (prevIndex < 0) {
          const remainder = themes.length % indexesPerItem;
          const lastGroupStart =
            remainder === 0 ? themes.length - indexesPerItem : themes.length - remainder;
          return lastGroupStart;
        }
        return prevIndex;
      });
    });

    const visibleThemes = useMemo(() => {
      if (currentIndex + indexesPerItem > themes.length) {
        return themes.slice(currentIndex);
      }
      return themes.slice(currentIndex, currentIndex + indexesPerItem);
    }, [currentIndex, themes, indexesPerItem]);

    const itemWidth = useMemo(() => {
      return visibleThemes.length < indexesPerItem
        ? `${100 / visibleThemes.length}%`
        : `${100 / indexesPerItem}%`;
    }, [visibleThemes.length, indexesPerItem]);

    const memoizedAnimation = useMemo(() => {
      return {
        initial: { x: 300, opacity: 0 },
        animate: { x: 0, opacity: 1 },
        exit: { x: -300, opacity: 0 },
        transition: { type: 'spring', stiffness: 300, damping: 30 }
      };
    }, []);

    useEffect(() => {
      setCurrentIndex(initialIndex);
    }, [themes.length]);

    return (
      <div
        ref={containerRef}
        className={cx('flex w-full items-center justify-between', styles.carouselContainer)}>
        <ThemeCarouselButton side="left" onClick={handlePrev} />

        <div className="relative h-full flex-1 overflow-hidden">
          <AnimatePresence mode="popLayout" initial={false}>
            <motion.div
              key={currentIndex}
              className="absolute flex h-full w-full items-center space-x-0.5 px-0.5"
              {...memoizedAnimation}>
              {visibleThemes.map((theme, idx) => (
                <div key={idx} className="h-full" style={{ width: itemWidth }}>
                  <ThemeCarouselItem theme={theme} onChange={onChange} showTitle={showTitle} />
                </div>
              ))}
            </motion.div>
          </AnimatePresence>
        </div>

        <ThemeCarouselButton onClick={handleNext} side="right" />
      </div>
    );
  }
);
ThemeCarousel.displayName = 'ThemeCarousel';

const ThemeCarouselItem: React.FC<{
  theme: IThemeCarouselItem;
  onChange: (theme: IThemeCarouselItem) => void;
  showTitle?: boolean;
}> = React.memo(({ theme, onChange, showTitle = true }) => {
  const { styles, cx } = useStyles();

  return (
    <div
      onClick={() => onChange(theme)}
      className={cx(
        'flex w-full items-center gap-1 px-2.5',
        styles.themeCarouselItem,
        theme.selected && 'selected'
      )}>
      <ThemeColorDots colors={theme.colors} />
      {showTitle && <span className="min-w-0 truncate text-sm">{theme.name}</span>}
    </div>
  );
});
ThemeCarouselItem.displayName = 'ThemeCarouselItem';

const ThemeCarouselButton: React.FC<{ onClick: () => void; side: 'left' | 'right' }> = React.memo(
  ({ onClick, side }) => {
    const { styles, cx } = useStyles();

    return (
      <div
        aria-label={side === 'left' ? 'Previous themes' : 'Next themes'}
        className={cx(styles.themeCarouselButton)}
        onClick={onClick}>
        <AppMaterialIcons icon={side === 'left' ? 'chevron_left' : 'chevron_right'} />
      </div>
    );
  }
);
ThemeCarouselButton.displayName = 'ThemeCarouselButton';

const useStyles = createStyles(({ css, token }) => ({
  themeCarouselButton: css`
    width: 20px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;

    &:hover {
      background-color: ${token.colorBgContainer};
    }
  `,

  themeCarouselItem: css`
    background-color: ${token.controlItemBgActive};
    transition: all 0.12s ease;
    border-radius: ${token.borderRadius}px;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: ${token.colorTextSecondary};

    &.selected {
      background-color: ${token.colorBgContainer};
      color: ${token.colorText};
      position: relative;
      z-index: 1;
      box-shadow: 0 0 0 0.75px ${token.colorBorder};

      .ball {
        box-shadow: 0 0 0 0.75px ${token.colorBgContainer};
      }
    }

    &:hover {
      background-color: ${token.colorBgContainer};
      color: ${token.colorText};
    }
  `,
  carouselContainer: css`
    height: ${token.controlHeight}px;
    border: 0.5px solid ${token.colorBorder};
    border-radius: ${token.borderRadius}px;
    background-color: ${token.controlItemBgActive};
    overflow: hidden;
  `
}));
