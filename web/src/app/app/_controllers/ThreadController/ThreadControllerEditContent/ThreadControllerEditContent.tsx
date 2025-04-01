import React, { useMemo, useState } from 'react';
import { IBusterThread } from '@/context/Threads/interfaces';
import { SidebarApplications } from './config';
import { AnimatePresence, motion } from 'framer-motion';
import { SidebarMetricApp } from './SidebarMetricApp/SidebarMetricApp';
import { SidebarStylingApp } from './SidebarStylingApp';
import { SidebarSQLApp } from './SidebarSQLApp';
import { EditContentTopApp } from './EditContentTopApp';
import { AppContent } from '@/app/app/_components/AppContent';

const SidebarApplicationsRecord: Record<
  SidebarApplications,
  React.FC<{
    showSkeletonLoader?: boolean;
    isReadOnly: boolean;
    threadId: string;
  }>
> = {
  [SidebarApplications.Copilot]: SidebarMetricApp,
  [SidebarApplications.Styling]: SidebarStylingApp,
  [SidebarApplications.SQL]: SidebarSQLApp
};

const MOTION_CONFIG = {
  initial: { opacity: 0, y: -10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 10 },
  transition: { duration: 0.125 }
};

export const ThreadControllerEditContent: React.FC<{
  threadId: IBusterThread['id'];
  minSize: number;
  isReadOnly: boolean;
}> = React.memo(({ threadId, minSize, isReadOnly }) => {
  const [selectedApplication, setSelectedApplication] = useState<SidebarApplications>(
    SidebarApplications.Copilot
  );
  const SelectedApplication = useMemo(
    () => SidebarApplicationsRecord[selectedApplication],
    [selectedApplication]
  );

  return (
    <div className="flex h-full flex-col" style={{ minWidth: minSize }}>
      <EditContentTopApp
        selectedApp={selectedApplication}
        setSelectedApp={setSelectedApplication}
        disabled={false}
        threadId={threadId}
        isReadOnly={isReadOnly}
      />

      <AppContent>
        <AnimatePresence mode="wait" initial={false}>
          <motion.div
            initial={MOTION_CONFIG.initial}
            animate={MOTION_CONFIG.animate}
            exit={MOTION_CONFIG.exit}
            transition={MOTION_CONFIG.transition}
            className="h-full"
            key={selectedApplication}>
            <SelectedApplication
              showSkeletonLoader={false}
              isReadOnly={isReadOnly}
              threadId={threadId}
            />
          </motion.div>
        </AnimatePresence>
      </AppContent>
    </div>
  );
});
ThreadControllerEditContent.displayName = 'ThreadControllerEditContent';
