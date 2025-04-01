import React from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { PulseLoader } from '../loaders';
import { createStyles } from 'antd-style';
import { AppMaterialIcons } from '../icons';

export const AppVerticalSteps: React.FC<{
  title: string | React.ReactNode;
  description: string | React.ReactNode;
  isCurrent: boolean;
  showSideLine: boolean;
  isLast: boolean;
  isCollapsedContent?: boolean;
  animateInitial?: boolean;
}> = ({
  animateInitial = true,
  isCollapsedContent = false,
  isLast,
  title,
  showSideLine,
  isCurrent,
  description
}) => {
  const { cx, styles } = useStyles();

  return (
    <AnimatePresence initial={animateInitial}>
      <motion.div
        initial={{ opacity: 0, height: 0 }}
        animate={{
          opacity: 1,
          height: 'auto',
          transition: {
            opacity: { duration: 0.2, delay: 0.05 },
            height: { duration: 0.2 }
          }
        }}
        transition={{ duration: 0.25 }}
        className={cx(`flex h-full space-x-4`)}>
        <div className="relative flex flex-col items-center justify-center pl-2">
          <div className="absolute top-0">
            {isCurrent ? (
              <PulseLoader size={9} />
            ) : (
              <div
                className={cx(
                  'flex items-center justify-center rounded-full p-1 text-sm',
                  styles.icon
                )}
                style={{
                  height: 8,
                  width: 8
                }}>
                <AppMaterialIcons className="mt-0" icon="check_circle" fill size={12} />
              </div>
            )}
          </div>

          <div
            className={cx('h-full w-[0.5px] group-hover:bg-[#808080]', styles.line)}
            style={{
              opacity: !showSideLine || (isLast && !description) ? 0 : 1
            }}
          />
        </div>
        <div className={`content -mt-1 w-full flex-grow space-y-1.5 ${isLast ? 'pb-0' : 'pb-5'}`}>
          <div className="flex space-x-2">
            <div>{title}</div>
          </div>

          <AnimatePresence initial={animateInitial}>
            <motion.div
              className="overflow-hidden"
              initial={{ opacity: 0 }}
              animate={{
                opacity: 1,
                height: 'auto'
              }}
              transition={{
                delay: 0.125,
                duration: 0.21,
                height: {
                  delay: 0
                }
              }}
              data-qa="description">
              {description}
            </motion.div>
          </AnimatePresence>
        </div>
      </motion.div>
    </AnimatePresence>
  );
};

const useStyles = createStyles(({ token, css }) => {
  return {
    line: {
      background: token.colorBorderSecondary
    },
    icon: {
      color: token.colorTextTertiary
    }
  };
});
